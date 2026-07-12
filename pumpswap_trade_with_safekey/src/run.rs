//! 买→休息→卖→休息 循环执行逻辑

use sol_trade_sdk::{
    common::nonce_cache::{fetch_nonce_info, DurableNonceInfo},
    swqos::SwqosType,
    trading::{
        core::params::{DexParamEnum, PumpSwapParams},
        factory::DexType,
    },
    AccountPolicy, BuyAmount, SellAmount, SimpleBuyParams, SimpleSellParams, SolanaTrade,
    TradeTokenType,
};
use solana_sdk::pubkey::Pubkey;

use crate::config::TradingConfig;
use crate::swqos::build_gas_fee_strategy;

/// 每轮：买入后休息秒数、卖出后休息秒数
const REST_SECS: u64 = 30;
/// 买→休息→卖→休息 的循环次数
const ROUNDS: u32 = 1;

pub const fn rest_secs() -> u64 {
    REST_SECS
}
pub const fn rounds() -> u32 {
    ROUNDS
}

pub struct PumpSwapRunConfig<'a> {
    pub trading_config: Option<&'a TradingConfig>,
    pub swqos_types: &'a [SwqosType],
    pub durable_nonce_buy: Option<DurableNonceInfo>,
    pub durable_nonce_sell: Option<DurableNonceInfo>,
    pub sol_lamports: u64,
    pub buy_slippage_bps: u64,
    pub sell_slippage_bps: u64,
}

pub async fn run_pumpswap_loop(
    client: &SolanaTrade,
    mint_pubkey: Pubkey,
    pool: Pubkey,
    run_config: PumpSwapRunConfig<'_>,
) -> anyhow::Result<()> {
    let PumpSwapRunConfig {
        trading_config,
        swqos_types,
        durable_nonce_buy,
        durable_nonce_sell,
        sol_lamports,
        buy_slippage_bps,
        sell_slippage_bps,
    } = run_config;
    if sol_lamports == 0 {
        anyhow::bail!("买入金额必须大于 0 lamports");
    }
    if buy_slippage_bps >= 10_000 || sell_slippage_bps >= 10_000 {
        anyhow::bail!("买入和卖出滑点必须小于 10000 bps");
    }
    for round in 1..=ROUNDS {
        println!("\n========== 第 {} / {} 轮 ==========", round, ROUNDS);

        let gas = build_gas_fee_strategy(trading_config, swqos_types);
        let pool_params =
            PumpSwapParams::from_pool_address_by_rpc(&client.infrastructure.rpc, &pool).await?;
        let mint_token_program = if pool_params.base_mint == mint_pubkey {
            pool_params.base_token_program
        } else if pool_params.quote_mint == mint_pubkey {
            pool_params.quote_token_program
        } else {
            anyhow::bail!("目标 mint {} 不属于 PumpSwap 池 {}", mint_pubkey, pool);
        };
        let balance_before = client
            .get_payer_token_balance_with_program(&mint_pubkey, &mint_token_program)
            .await?;

        println!("[2] 买入（同时发往所有已配置 SWQoS）...");
        let buy_amount = BuyAmount::WithMaxInput {
            quote_amount: sol_lamports,
        };
        let buy_params = if let Some(nonce) = durable_nonce_buy.clone() {
            SimpleBuyParams::with_durable_nonce(
                DexType::PumpSwap,
                TradeTokenType::SOL,
                mint_pubkey,
                buy_amount,
                DexParamEnum::PumpSwap(pool_params.clone()),
                nonce,
                gas.clone(),
            )
        } else {
            SimpleBuyParams::new(
                DexType::PumpSwap,
                TradeTokenType::SOL,
                mint_pubkey,
                buy_amount,
                DexParamEnum::PumpSwap(pool_params.clone()),
                client.infrastructure.rpc.get_latest_blockhash().await?,
                gas.clone(),
            )
        }
        .slippage_basis_points(buy_slippage_bps)
        .account_policy(AccountPolicy::Auto)
        .wait_tx_confirmed(true)
        .wait_for_all_submits(false);
        let (ok, sigs, err, _) = client.buy_simple(buy_params).await?;
        if !ok {
            let e = err
                .as_ref()
                .map(|e| e.to_string())
                .unwrap_or_else(|| "unknown".to_string());
            anyhow::bail!("第 {} 轮买入失败: {} | sigs: {:?}", round, e, sigs);
        }
        println!("    买入成功: {:?}", sigs);

        println!("[3] 买入后休息 {} 秒...", REST_SECS);
        tokio::time::sleep(std::time::Duration::from_secs(REST_SECS)).await;

        let balance_after = client
            .get_payer_token_balance_with_program(&mint_pubkey, &mint_token_program)
            .await?;
        let position_amount = balance_after.checked_sub(balance_before).ok_or_else(|| {
            anyhow::anyhow!(
                "第 {} 轮：买入后余额 {} 小于买入前余额 {}，拒绝卖出旧持仓",
                round,
                balance_after,
                balance_before
            )
        })?;
        if position_amount == 0 {
            anyhow::bail!("第 {} 轮：买入确认后余额没有增加，拒绝卖出", round);
        }
        println!("[4] 仅卖出本轮买入的 {} tokens...", position_amount);

        let durable_nonce_sell_fresh = if let Some(ref info) = durable_nonce_sell {
            if let Some(nonce_account) = info.nonce_account {
                match fetch_nonce_info(&client.infrastructure.rpc, nonce_account).await {
                    Some(fresh) => Some(fresh),
                    None => {
                        anyhow::bail!(
                            "无法获取 durable nonce 账户 {} 的最新状态，交易可能失败",
                            nonce_account
                        );
                    }
                }
            } else {
                anyhow::bail!("durable nonce 配置无效：缺少 nonce_account");
            }
        } else {
            None
        };

        let pool_params_sell =
            PumpSwapParams::from_pool_address_by_rpc(&client.infrastructure.rpc, &pool).await?;

        let sell_amount = SellAmount::ExactInput(position_amount);
        let sell_params = if let Some(nonce) = durable_nonce_sell_fresh {
            SimpleSellParams::with_durable_nonce(
                DexType::PumpSwap,
                TradeTokenType::SOL,
                mint_pubkey,
                sell_amount,
                DexParamEnum::PumpSwap(pool_params_sell),
                nonce,
                gas,
            )
        } else {
            SimpleSellParams::new(
                DexType::PumpSwap,
                TradeTokenType::SOL,
                mint_pubkey,
                sell_amount,
                DexParamEnum::PumpSwap(pool_params_sell),
                client.infrastructure.rpc.get_latest_blockhash().await?,
                gas,
            )
        };
        let sell_params = sell_params
            .slippage_basis_points(sell_slippage_bps)
            .account_policy(AccountPolicy::Auto)
            .wait_tx_confirmed(true)
            .wait_for_all_submits(false);
        let (ok, sigs, err, _) = client.sell_simple(sell_params).await?;
        if !ok {
            let e = err
                .as_ref()
                .map(|e| e.to_string())
                .unwrap_or_else(|| "unknown".to_string());
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
