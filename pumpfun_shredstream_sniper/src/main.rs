//! PumpFun ShredStream sniper example.
//!
//! Monitors PumpFun outer instructions through sol-parser-sdk ShredStream, buys once on
//! the selected event, waits HOLD_SECONDS, then sells the full token balance.

use std::{
    collections::HashMap,
    str::FromStr,
    sync::Arc,
    time::{Duration, Instant},
};

use anyhow::{Context, Result};
use sol_parser_sdk::{
    core::events::{PumpFunCreateTokenEvent, PumpFunCreateV2TokenEvent, PumpFunTradeEvent},
    shredstream::{ShredStreamClient, ShredStreamConfig},
    DexEvent,
};
use sol_trade_sdk::{
    common::{
        fast_fn::get_associated_token_address_with_program_id_fast_use_seed, GasFeeStrategy,
        TradeConfig,
    },
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
    shredstream_endpoint: String,
    target_mint: Option<Pubkey>,
    require_created_buy: bool,
    buy_lamports: u64,
    buy_slippage_bps: u64,
    sell_slippage_bps: u64,
    hold_seconds: u64,
    wait_tx_confirmed: bool,
}

#[derive(Clone)]
struct CreatedMintContext {
    creator: Pubkey,
    bonding_curve: Pubkey,
    associated_bonding_curve: Option<Pubkey>,
    token_program: Pubkey,
    is_mayhem_mode: bool,
    is_cashback_coin: bool,
    observed_at: Instant,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    let settings = Settings::from_env()?;
    println!("PumpFun ShredStream sniper started");
    println!("  wallet: {}", settings.payer.pubkey());
    println!("  rpc: {}", settings.rpc_url);
    println!("  shredstream: {}", settings.shredstream_endpoint);
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

    let client = ShredStreamClient::new_with_config(
        settings.shredstream_endpoint.clone(),
        ShredStreamConfig::low_latency(),
    )
    .await
    .map_err(|err| anyhow::anyhow!("failed to create ShredStream client: {}", err))?;
    let queue = client
        .subscribe()
        .await
        .map_err(|err| anyhow::anyhow!("failed to subscribe ShredStream events: {}", err))?;

    println!("subscription ready, waiting for one matching PumpFun buy event...");
    let mut created_mints: HashMap<Pubkey, CreatedMintContext> = HashMap::new();
    let mut spin_count = 0u32;

    loop {
        if let Some(event) = queue.pop() {
            spin_count = 0;
            prune_created_mints(&mut created_mints);
            match event {
                DexEvent::PumpFunCreate(e) => {
                    remember_create(&mut created_mints, e);
                }
                DexEvent::PumpFunCreateV2(e) => {
                    remember_create_v2(&mut created_mints, e);
                }
                DexEvent::PumpFunTrade(mut e)
                | DexEvent::PumpFunBuy(mut e)
                | DexEvent::PumpFunBuyExactSolIn(mut e) => {
                    if let Some(ctx) = created_mints.get(&e.mint) {
                        enrich_trade_from_create(&mut e, ctx);
                    }
                    if select_trade_event(&e, settings.target_mint, settings.require_created_buy) {
                        println!(
                            "matched event: mint={}, sig={}, slot={}, ix={}",
                            e.mint, e.metadata.signature, e.metadata.slot, e.ix_name
                        );
                        execute_buy_hold_sell(settings.clone(), e).await?;
                        client.stop().await;
                        break;
                    }
                }
                _ => {}
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
        let shredstream_endpoint = env_string("SHREDSTREAM_ENDPOINT")
            .unwrap_or_else(|| "http://127.0.0.1:10800".to_string());
        let target_mint = env_string("TARGET_MINT")
            .map(|s| Pubkey::from_str(&s))
            .transpose()
            .context("TARGET_MINT is not a valid pubkey")?;
        let buy_sol = env_f64("BUY_SOL_AMOUNT", 0.01)?;

        Ok(Self {
            payer,
            rpc_url,
            shredstream_endpoint,
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

fn remember_create(
    created_mints: &mut HashMap<Pubkey, CreatedMintContext>,
    event: PumpFunCreateTokenEvent,
) {
    created_mints.insert(
        event.mint,
        CreatedMintContext {
            creator: event.creator,
            bonding_curve: event.bonding_curve,
            associated_bonding_curve: None,
            token_program: event.token_program,
            is_mayhem_mode: event.is_mayhem_mode,
            is_cashback_coin: event.is_cashback_enabled,
            observed_at: Instant::now(),
        },
    );
}

fn remember_create_v2(
    created_mints: &mut HashMap<Pubkey, CreatedMintContext>,
    event: PumpFunCreateV2TokenEvent,
) {
    created_mints.insert(
        event.mint,
        CreatedMintContext {
            creator: event.creator,
            bonding_curve: event.bonding_curve,
            associated_bonding_curve: Some(event.associated_bonding_curve),
            token_program: event.token_program,
            is_mayhem_mode: event.is_mayhem_mode,
            is_cashback_coin: event.is_cashback_enabled,
            observed_at: Instant::now(),
        },
    );
}

fn enrich_trade_from_create(event: &mut PumpFunTradeEvent, ctx: &CreatedMintContext) {
    if event.creator == Pubkey::default() {
        event.creator = ctx.creator;
    }
    if event.bonding_curve == Pubkey::default() {
        event.bonding_curve = ctx.bonding_curve;
    }
    if event.token_program == Pubkey::default() {
        event.token_program = ctx.token_program;
    }
    if event.associated_bonding_curve == Pubkey::default() {
        event.associated_bonding_curve = ctx.associated_bonding_curve.unwrap_or_else(|| {
            get_associated_token_address_with_program_id_fast_use_seed(
                &ctx.bonding_curve,
                &event.mint,
                &ctx.token_program,
                true,
            )
        });
    }
    event.mayhem_mode = event.mayhem_mode || ctx.is_mayhem_mode;
    event.is_cashback_coin = event.is_cashback_coin || ctx.is_cashback_coin;
}

fn prune_created_mints(created_mints: &mut HashMap<Pubkey, CreatedMintContext>) {
    let now = Instant::now();
    created_mints.retain(|_, ctx| now.duration_since(ctx.observed_at) < Duration::from_secs(60));
}

fn select_trade_event(
    event: &PumpFunTradeEvent,
    target_mint: Option<Pubkey>,
    require_created_buy: bool,
) -> bool {
    if !event.is_buy {
        return false;
    }
    if require_created_buy && !event.is_created_buy {
        return false;
    }
    if let Some(target) = target_mint {
        if event.mint != target {
            return false;
        }
    }
    true
}

async fn execute_buy_hold_sell(settings: Settings, event: PumpFunTradeEvent) -> Result<()> {
    let client = create_trade_client(&settings).await?;
    let gas_fee_strategy = default_gas_fee_strategy();

    validate_trade_event_for_buy(&event)?;

    let recent_blockhash = client.infrastructure.rpc.get_latest_blockhash().await?;
    let buy_params = sol_trade_sdk::TradeBuyParams {
        dex_type: DexType::PumpFun,
        input_token_type: TradeTokenType::SOL,
        mint: event.mint,
        input_token_amount: settings.buy_lamports,
        slippage_basis_points: Some(settings.buy_slippage_bps),
        recent_blockhash: Some(recent_blockhash),
        extension_params: DexParamEnum::PumpFun(build_buy_params(&client, &event).await?),
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
    let sell_params = sol_trade_sdk::TradeSellParams {
        dex_type: DexType::PumpFun,
        output_token_type: TradeTokenType::SOL,
        mint: event.mint,
        input_token_amount: amount_token,
        slippage_basis_points: Some(settings.sell_slippage_bps),
        recent_blockhash: Some(sell_recent_blockhash),
        with_tip: true,
        extension_params: DexParamEnum::PumpFun(build_fresh_sell_params(&client, &event).await?),
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

async fn build_buy_params(
    client: &SolanaTrade,
    event: &PumpFunTradeEvent,
) -> Result<PumpFunParams> {
    if event.is_created_buy && event.creator != Pubkey::default() {
        let max_sol_cost = event.sol_amount.saturating_add(event.sol_amount / 10);
        return Ok(PumpFunParams::from_dev_trade(
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
        ));
    }

    PumpFunParams::from_mint_by_rpc(&client.infrastructure.rpc, &event.mint)
        .await
        .with_context(|| {
            "failed to build buy params from ShredStream event; set REQUIRE_CREATED_BUY=true or wait until RPC can see the mint"
        })
}

async fn build_fresh_sell_params(
    client: &SolanaTrade,
    event: &PumpFunTradeEvent,
) -> Result<PumpFunParams> {
    let params = PumpFunParams::from_mint_by_rpc(&client.infrastructure.rpc, &event.mint)
        .await
        .context("failed to refresh PumpFun params by RPC before sell")?;
    Ok(params.with_creator_vault(event.creator_vault))
}

fn validate_trade_event_for_buy(event: &PumpFunTradeEvent) -> Result<()> {
    if event.bonding_curve == Pubkey::default()
        || event.associated_bonding_curve == Pubkey::default()
        || event.token_program == Pubkey::default()
    {
        anyhow::bail!("ShredStream event is missing required PumpFun accounts");
    }
    if event.is_created_buy && event.creator == Pubkey::default() {
        anyhow::bail!("ShredStream created-buy event is missing creator context");
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
