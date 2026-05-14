//! PumpFun gRPC sniper example.
//!
//! Monitors PumpFun trades through sol-parser-sdk Yellowstone gRPC, buys once on the
//! selected event, waits HOLD_SECONDS, then sells the full token balance.

use std::{str::FromStr, sync::Arc, time::Duration};

use anyhow::{Context, Result};
use sol_parser_sdk::{
    core::events::PumpFunTradeEvent,
    grpc::{
        AccountFilter, ClientConfig, EventType, EventTypeFilter, OrderMode, Protocol,
        TransactionFilter, YellowstoneGrpc,
    },
    DexEvent,
};
use sol_trade_sdk::{
    common::{GasFeeStrategy, TradeConfig},
    swqos::SwqosConfig,
    trading::{
        core::params::{DexParamEnum, PumpFunParams},
        factory::DexType,
    },
    SolanaTrade, TradeTokenType,
};
use solana_commitment_config::CommitmentConfig;
use solana_sdk::{
    native_token::LAMPORTS_PER_SOL, pubkey::Pubkey, signature::Keypair, signer::Signer,
};

#[derive(Clone)]
struct Settings {
    payer: Arc<Keypair>,
    rpc_url: String,
    grpc_endpoint: String,
    grpc_auth_token: Option<String>,
    target_mint: Option<Pubkey>,
    require_created_buy: bool,
    buy_lamports: u64,
    buy_slippage_bps: u64,
    sell_slippage_bps: u64,
    hold_seconds: u64,
    wait_tx_confirmed: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    let _ = rustls::crypto::ring::default_provider().install_default();

    let settings = Settings::from_env()?;
    println!("PumpFun gRPC sniper started");
    println!("  wallet: {}", settings.payer.pubkey());
    println!("  rpc: {}", settings.rpc_url);
    println!("  grpc: {}", settings.grpc_endpoint);
    println!(
        "  buy: {} lamports ({:.6} SOL), hold: {}s",
        settings.buy_lamports,
        settings.buy_lamports as f64 / LAMPORTS_PER_SOL as f64,
        settings.hold_seconds
    );
    if let Some(mint) = settings.target_mint {
        println!("  target mint: {}", mint);
    }
    println!("  require_created_buy: {}", settings.require_created_buy);

    let config = ClientConfig {
        enable_metrics: false,
        connection_timeout_ms: 10_000,
        request_timeout_ms: 30_000,
        enable_tls: true,
        order_mode: OrderMode::Unordered,
        ..Default::default()
    };
    let grpc = YellowstoneGrpc::new_with_config(
        settings.grpc_endpoint.clone(),
        settings.grpc_auth_token.clone(),
        config,
    )
    .map_err(|err| anyhow::anyhow!("failed to create Yellowstone gRPC client: {}", err))?;

    let protocols = vec![Protocol::PumpFun];
    let transaction_filter = TransactionFilter::for_protocols(&protocols);
    let account_filter = AccountFilter::for_protocols(&protocols);
    let event_filter = EventTypeFilter::include_only(vec![
        EventType::PumpFunBuy,
        EventType::PumpFunBuyExactSolIn,
        EventType::PumpFunTrade,
    ]);

    let queue = grpc
        .subscribe_dex_events(
            vec![transaction_filter],
            vec![account_filter],
            Some(event_filter),
        )
        .await
        .map_err(|err| anyhow::anyhow!("failed to subscribe gRPC events: {}", err))?;

    println!("subscription ready, waiting for one matching PumpFun buy event...");
    let mut spin_count = 0u32;
    loop {
        if let Some(event) = queue.pop() {
            spin_count = 0;
            if let Some(event) =
                select_trade_event(event, settings.target_mint, settings.require_created_buy)
            {
                println!(
                    "matched event: mint={}, sig={}, slot={}, ix={}",
                    event.mint, event.metadata.signature, event.metadata.slot, event.ix_name
                );
                execute_buy_hold_sell(settings.clone(), event).await?;
                break;
            }
        } else {
            spin_count += 1;
            if spin_count < 1_000 {
                std::hint::spin_loop();
            } else {
                tokio::task::yield_now().await;
                spin_count = 0;
            }
        }
    }

    Ok(())
}

impl Settings {
    fn from_env() -> Result<Self> {
        let payer = Arc::new(load_keypair_from_env()?);
        let rpc_url = env_string("RPC_URL")
            .or_else(|| env_string("SOLANA_RPC_URL"))
            .unwrap_or_else(|| "https://api.mainnet-beta.solana.com".to_string());
        let grpc_endpoint = env_string("GRPC_ENDPOINT")
            .unwrap_or_else(|| "https://solana-yellowstone-grpc.publicnode.com:443".to_string());
        let grpc_auth_token = env_string("GRPC_AUTH_TOKEN");
        let target_mint = env_string("TARGET_MINT")
            .map(|s| Pubkey::from_str(&s))
            .transpose()
            .context("TARGET_MINT is not a valid pubkey")?;
        let buy_sol = env_f64("BUY_SOL_AMOUNT", 0.01)?;

        Ok(Self {
            payer,
            rpc_url,
            grpc_endpoint,
            grpc_auth_token,
            target_mint,
            require_created_buy: env_bool("REQUIRE_CREATED_BUY", true),
            buy_lamports: (buy_sol * LAMPORTS_PER_SOL as f64).round() as u64,
            buy_slippage_bps: env_u64("BUY_SLIPPAGE_BPS", 300)?,
            sell_slippage_bps: env_u64("SELL_SLIPPAGE_BPS", 9980)?,
            hold_seconds: env_u64("HOLD_SECONDS", 3)?,
            wait_tx_confirmed: env_bool("WAIT_TX_CONFIRMED", true),
        })
    }
}

fn select_trade_event(
    event: DexEvent,
    target_mint: Option<Pubkey>,
    require_created_buy: bool,
) -> Option<PumpFunTradeEvent> {
    let event = match event {
        DexEvent::PumpFunBuy(e) | DexEvent::PumpFunBuyExactSolIn(e) | DexEvent::PumpFunTrade(e) => {
            e
        }
        _ => return None,
    };
    if !event.is_buy {
        return None;
    }
    if require_created_buy && !event.is_created_buy {
        return None;
    }
    if let Some(target) = target_mint {
        if event.mint != target {
            return None;
        }
    }
    Some(event)
}

async fn execute_buy_hold_sell(settings: Settings, event: PumpFunTradeEvent) -> Result<()> {
    let client = create_trade_client(&settings).await?;
    let gas_fee_strategy = default_gas_fee_strategy();

    validate_trade_event(&event)?;

    let recent_blockhash = client.infrastructure.rpc.get_latest_blockhash().await?;
    let buy_params = sol_trade_sdk::TradeBuyParams {
        dex_type: DexType::PumpFun,
        input_token_type: TradeTokenType::SOL,
        mint: event.mint,
        input_token_amount: settings.buy_lamports,
        slippage_basis_points: Some(settings.buy_slippage_bps),
        recent_blockhash: Some(recent_blockhash),
        extension_params: DexParamEnum::PumpFun(build_buy_params_from_event(&event)),
        address_lookup_table_account: None,
        wait_tx_confirmed: settings.wait_tx_confirmed,
        create_input_token_ata: false,
        close_input_token_ata: false,
        create_mint_ata: true,
        durable_nonce: None,
        fixed_output_token_amount: None,
        gas_fee_strategy: gas_fee_strategy.clone(),
        simulate: false,
        use_exact_sol_amount: Some(true),
        grpc_recv_us: Some(event.metadata.grpc_recv_us),
    };

    println!("buying {}...", event.mint);
    let (ok, sigs, err, _) = client.buy(buy_params).await?;
    if !ok {
        let message = err
            .map(|e| e.to_string())
            .unwrap_or_else(|| "unknown buy error".to_string());
        anyhow::bail!("buy failed: {} | sigs: {:?}", message, sigs);
    }
    println!("buy submitted: {:?}", sigs);

    println!("holding {} seconds before sell...", settings.hold_seconds);
    tokio::time::sleep(Duration::from_secs(settings.hold_seconds)).await;

    let amount_token = client
        .get_payer_token_balance_with_program(&event.mint, &event.token_program)
        .await
        .context("failed to read token balance before sell")?;
    if amount_token == 0 {
        anyhow::bail!("token balance is zero, nothing to sell");
    }

    let sell_recent_blockhash = client.infrastructure.rpc.get_latest_blockhash().await?;
    let sell_extension = build_fresh_sell_params(&client, &event).await;
    let sell_params = sol_trade_sdk::TradeSellParams {
        dex_type: DexType::PumpFun,
        output_token_type: TradeTokenType::SOL,
        mint: event.mint,
        input_token_amount: amount_token,
        slippage_basis_points: Some(settings.sell_slippage_bps),
        recent_blockhash: Some(sell_recent_blockhash),
        with_tip: true,
        extension_params: DexParamEnum::PumpFun(sell_extension),
        address_lookup_table_account: None,
        wait_tx_confirmed: settings.wait_tx_confirmed,
        create_output_token_ata: true,
        close_output_token_ata: false,
        close_mint_token_ata: false,
        durable_nonce: None,
        fixed_output_token_amount: None,
        gas_fee_strategy,
        simulate: false,
        grpc_recv_us: None,
    };

    println!("selling {} tokens...", amount_token);
    let (ok, sigs, err, _) = client.sell(sell_params).await?;
    if !ok {
        let message = err
            .map(|e| e.to_string())
            .unwrap_or_else(|| "unknown sell error".to_string());
        anyhow::bail!("sell failed: {} | sigs: {:?}", message, sigs);
    }
    println!("sell submitted: {:?}", sigs);
    println!("done");
    Ok(())
}

async fn create_trade_client(settings: &Settings) -> Result<SolanaTrade> {
    let swqos_configs = vec![SwqosConfig::Default(settings.rpc_url.clone())];
    let trade_config = TradeConfig::builder(
        settings.rpc_url.clone(),
        swqos_configs,
        CommitmentConfig::confirmed(),
    )
    .create_wsol_ata_on_startup(false)
    .use_seed_optimize(true)
    .log_enabled(true)
    .build();
    Ok(SolanaTrade::new(settings.payer.clone(), trade_config).await)
}

fn build_buy_params_from_event(event: &PumpFunTradeEvent) -> PumpFunParams {
    if event.is_created_buy {
        let max_sol_cost = event.sol_amount.saturating_add(event.sol_amount / 10);
        PumpFunParams::from_dev_trade(
            event.mint,
            event.token_amount,
            max_sol_cost,
            event.creator,
            event.bonding_curve,
            event.associated_bonding_curve,
            event.creator_vault,
            None,
            event.fee_recipient,
            event.token_program,
            event.is_cashback_coin,
            Some(event.mayhem_mode),
        )
    } else {
        build_trade_params_from_event(event, None)
    }
}

fn build_trade_params_from_event(
    event: &PumpFunTradeEvent,
    close_token_account_when_sell: Option<bool>,
) -> PumpFunParams {
    PumpFunParams::from_trade(
        event.bonding_curve,
        event.associated_bonding_curve,
        event.mint,
        event.creator,
        event.creator_vault,
        event.virtual_token_reserves,
        event.virtual_sol_reserves,
        event.real_token_reserves,
        event.real_sol_reserves,
        close_token_account_when_sell,
        event.fee_recipient,
        event.token_program,
        event.is_cashback_coin,
        Some(event.mayhem_mode),
    )
}

async fn build_fresh_sell_params(client: &SolanaTrade, event: &PumpFunTradeEvent) -> PumpFunParams {
    match PumpFunParams::from_mint_by_rpc(&client.infrastructure.rpc, &event.mint).await {
        Ok(params) => params.with_creator_vault(event.creator_vault),
        Err(err) => {
            eprintln!(
                "warning: failed to refresh PumpFun params by RPC before sell: {}; using event params",
                err
            );
            build_trade_params_from_event(event, Some(true))
        }
    }
}

fn validate_trade_event(event: &PumpFunTradeEvent) -> Result<()> {
    if event.bonding_curve == Pubkey::default()
        || event.associated_bonding_curve == Pubkey::default()
        || event.token_program == Pubkey::default()
    {
        anyhow::bail!("stream event is missing required PumpFun accounts");
    }
    if event.is_created_buy && event.creator == Pubkey::default() {
        anyhow::bail!("created-buy event is missing creator");
    }
    Ok(())
}

fn default_gas_fee_strategy() -> GasFeeStrategy {
    let gas = GasFeeStrategy::new();
    gas.set_global_fee_strategy(150_000, 150_000, 500_000, 500_000, 0.001, 0.001);
    gas
}

fn load_keypair_from_env() -> Result<Keypair> {
    let private_key = env_string("PRIVATE_KEY").context("PRIVATE_KEY is required")?;
    load_keypair_from_string(&private_key)
}

fn load_keypair_from_string(value: &str) -> Result<Keypair> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        anyhow::bail!("PRIVATE_KEY is empty");
    }
    if let Ok(bytes) = serde_json::from_str::<Vec<u8>>(trimmed) {
        if bytes.len() == 64 {
            return Keypair::try_from(bytes.as_slice()).context("invalid 64-byte PRIVATE_KEY");
        }
    }
    if trimmed.len() > 50 && !trimmed.starts_with('[') && !trimmed.starts_with('{') {
        return std::panic::catch_unwind(|| Keypair::from_base58_string(trimmed))
            .map_err(|_| anyhow::anyhow!("invalid base58 PRIVATE_KEY"));
    }
    anyhow::bail!("PRIVATE_KEY must be base58 or a 64-byte JSON array")
}

fn env_string(key: &str) -> Option<String> {
    std::env::var(key)
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

fn env_bool(key: &str, default: bool) -> bool {
    env_string(key)
        .map(|s| matches!(s.to_ascii_lowercase().as_str(), "1" | "true" | "yes" | "y"))
        .unwrap_or(default)
}

fn env_u64(key: &str, default: u64) -> Result<u64> {
    env_string(key)
        .map(|s| {
            s.parse::<u64>()
                .with_context(|| format!("{} must be u64", key))
        })
        .unwrap_or(Ok(default))
}

fn env_f64(key: &str, default: f64) -> Result<f64> {
    env_string(key)
        .map(|s| {
            s.parse::<f64>()
                .with_context(|| format!("{} must be f64", key))
        })
        .unwrap_or(Ok(default))
}
