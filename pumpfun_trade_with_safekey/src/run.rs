//! 买→休息→卖→休息 循环执行逻辑（PumpFun 内盘 bonding curve）。
//! 每轮按钱包该 mint 的**全部**代币余额卖出（含买入前已有持仓），与 `pumpswap_trade` 示例一致。

use sol_trade_sdk::{
    common::{
        fast_fn::get_associated_token_address_with_program_id_fast_use_seed,
        nonce_cache::{fetch_nonce_info, DurableNonceInfo},
    },
    swqos::SwqosType,
    trading::{
        core::params::{DexParamEnum, PumpFunParams},
        factory::DexType,
    },
    SolanaTrade, TradeTokenType,
};
use solana_sdk::pubkey::Pubkey;

use crate::config::TradingConfig;
use crate::swqos::build_gas_fee_strategy;

const REST_SECS: u64 = 30;
const ROUNDS: u32 = 1;

pub const fn rest_secs() -> u64 { REST_SECS }
pub const fn rounds() -> u32 { ROUNDS }

pub async fn run_pumpfun_loop(
    client: &SolanaTrade,
    mint_pubkey: Pubkey,
    trading_config: Option<&TradingConfig>,
    swqos_types: &[SwqosType],
    durable_nonce_buy: Option<DurableNonceInfo>,
    durable_nonce_sell: Option<DurableNonceInfo>,
    sol_lamports: u64,
    buy_slippage_bps: u64,
    sell_slippage_bps: u64,
) -> anyhow::Result<()> {
    let payer_pubkey = client.get_payer_pubkey();
    let use_seed = true;

    for round in 1..=ROUNDS {
        println!("\n========== 第 {} / {} 轮 ==========", round, ROUNDS);

        let gas = build_gas_fee_strategy(trading_config, swqos_types);
        let recent_blockhash = client.infrastructure.rpc.get_latest_blockhash().await?;
        let pump_params =
            PumpFunParams::from_mint_by_rpc(&client.infrastructure.rpc, &mint_pubkey).await?;

        let mint_ata = get_associated_token_address_with_program_id_fast_use_seed(
            &payer_pubkey,
            &mint_pubkey,
            &pump_params.token_program,
            use_seed,
        );
        let mint_ata_exists = client.infrastructure.rpc.get_account(&mint_ata).await.is_ok();
        let create_mint_ata = !mint_ata_exists;
        if round == 1 {
            if mint_ata_exists {
                println!("[1c] 代币 ATA 已存在 ({})，买入时将不再创建", mint_ata);
            } else {
                println!("[1c] 代币 ATA 不存在，买入时将按需创建");
            }
        }

        println!("[2] 买入（PumpFun 内盘，同时发往所有已配置 SWQoS）...");
        let buy_params = sol_trade_sdk::TradeBuyParams {
            dex_type: DexType::PumpFun,
            input_token_type: TradeTokenType::SOL,
            mint: mint_pubkey,
            input_token_amount: sol_lamports,
            slippage_basis_points: Some(buy_slippage_bps),
            recent_blockhash: Some(recent_blockhash),
            extension_params: DexParamEnum::PumpFun(pump_params.clone()),
            address_lookup_table_account: None,
            wait_tx_confirmed: false,
            create_input_token_ata: false,
            close_input_token_ata: false,
            create_mint_ata,
            durable_nonce: durable_nonce_buy.clone(),
            fixed_output_token_amount: None,
            gas_fee_strategy: gas.clone(),
            simulate: false,
            use_exact_sol_amount: None,
            grpc_recv_us: None,
        };
        let (ok, sigs, err, _) = client.buy(buy_params).await?;
        if !ok {
            let e = err.as_ref().map(|e| e.to_string()).unwrap_or_else(|| "unknown".to_string());
            anyhow::bail!("第 {} 轮买入失败: {} | sigs: {:?}", round, e, sigs);
        }
        println!("    买入成功: {:?}", sigs);

        println!("[3] 买入后休息 {} 秒...", REST_SECS);
        tokio::time::sleep(std::time::Duration::from_secs(REST_SECS)).await;

        let pump_for_balance =
            PumpFunParams::from_mint_by_rpc(&client.infrastructure.rpc, &mint_pubkey).await?;
        let balance = client
            .get_payer_token_balance_with_program(&mint_pubkey, &pump_for_balance.token_program)
            .await?;
        if balance == 0 {
            anyhow::bail!("第 {} 轮：代币余额为 0，无法卖出", round);
        }
        println!("[4] 卖出 {} tokens...", balance);

        let durable_nonce_sell_fresh = if let Some(ref info) = durable_nonce_sell {
            if let Some(nonce_account) = info.nonce_account {
                match fetch_nonce_info(&client.infrastructure.rpc, nonce_account).await {
                    Some(fresh) => Some(fresh),
                    None => {
                        anyhow::bail!("无法获取 durable nonce 账户 {} 的最新状态，交易可能失败", nonce_account);
                    }
                }
            } else {
                anyhow::bail!("durable nonce 配置无效：缺少 nonce_account");
            }
        } else {
            None
        };

        let recent_blockhash_sell = client.infrastructure.rpc.get_latest_blockhash().await?;
        let pump_params_sell =
            PumpFunParams::from_mint_by_rpc(&client.infrastructure.rpc, &mint_pubkey).await?;

        let sell_params = sol_trade_sdk::TradeSellParams {
            dex_type: DexType::PumpFun,
            output_token_type: TradeTokenType::SOL,
            mint: mint_pubkey,
            input_token_amount: balance,
            slippage_basis_points: Some(sell_slippage_bps),
            recent_blockhash: Some(recent_blockhash_sell),
            with_tip: true,
            extension_params: DexParamEnum::PumpFun(pump_params_sell),
            address_lookup_table_account: None,
            wait_tx_confirmed: false,
            create_output_token_ata: true,
            close_output_token_ata: false,
            close_mint_token_ata: false,
            grpc_recv_us: None,
            durable_nonce: durable_nonce_sell_fresh,
            fixed_output_token_amount: None,
            gas_fee_strategy: gas,
            simulate: false,
        };
        let (ok, sigs, err, _) = client.sell(sell_params).await?;
        if !ok {
            let e = err.as_ref().map(|e| e.to_string()).unwrap_or_else(|| "unknown".to_string());
            anyhow::bail!("第 {} 轮卖出失败: {} | sigs: {:?}", round, e, sigs);
        }
        println!("    卖出成功: {:?}", sigs);

        if round < ROUNDS {
            println!("[5] 卖出后休息 {} 秒，进入下一轮...", REST_SECS);
            tokio::time::sleep(std::time::Duration::from_secs(REST_SECS)).await;
        }
    }

    println!("\n=== {} 轮全部完成 ===", ROUNDS);
    Ok(())
}
