//! PumpFun ShredStream sniper example.
//!
//! Monitors PumpFun outer instructions through sol-parser-sdk ShredStream, buys once on
//! the selected event, waits HOLD_SECONDS, then sells only the balance added by this run.

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
        clock::now_micros, fast_fn::get_associated_token_address_with_program_id_fast_use_seed,
        GasFeeStrategy, SolanaRpcClient, TradeConfig,
    },
    swqos::SwqosConfig,
    trading::{
        core::params::{DexParamEnum, PumpFunParams},
        factory::DexType,
    },
    AccountPolicy, BuyAmount, SellAmount, SimpleBuyParams, SimpleSellParams, SolanaTrade,
    TradeTokenType,
};
use solana_commitment_config::CommitmentConfig;
use solana_sdk::{
    hash::Hash, native_token::LAMPORTS_PER_SOL, pubkey::Pubkey, signature::Keypair, signer::Signer,
};
use tokio::sync::watch;

const BLOCKHASH_REFRESH_INTERVAL: Duration = Duration::from_millis(400);
const MAX_BLOCKHASH_AGE: Duration = Duration::from_secs(20);

#[derive(Clone, Copy)]
enum BuyMode {
    WithMaxInput,
    ExactInput,
}

impl BuyMode {
    fn amount(self, quote_amount: u64) -> BuyAmount {
        match self {
            Self::WithMaxInput => BuyAmount::WithMaxInput { quote_amount },
            Self::ExactInput => BuyAmount::ExactInput(quote_amount),
        }
    }
}

#[derive(Clone)]
struct CachedBlockhash {
    hash: Hash,
    fetched_at: Instant,
}

#[derive(Clone)]
struct BlockhashCache {
    receiver: watch::Receiver<CachedBlockhash>,
}

impl BlockhashCache {
    async fn start(rpc: Arc<SolanaRpcClient>) -> Result<Self> {
        let initial = CachedBlockhash {
            hash: rpc.get_latest_blockhash().await?,
            fetched_at: Instant::now(),
        };
        let (sender, receiver) = watch::channel(initial);
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(BLOCKHASH_REFRESH_INTERVAL);
            interval.tick().await;
            loop {
                interval.tick().await;
                match rpc.get_latest_blockhash().await {
                    Ok(hash) => {
                        if sender
                            .send(CachedBlockhash {
                                hash,
                                fetched_at: Instant::now(),
                            })
                            .is_err()
                        {
                            break;
                        }
                    }
                    Err(err) => eprintln!("warning: blockhash refresh failed: {err}"),
                }
            }
        });
        Ok(Self { receiver })
    }

    fn latest(&self) -> Result<Hash> {
        let cached = self.receiver.borrow().clone();
        if cached.fetched_at.elapsed() > MAX_BLOCKHASH_AGE {
            anyhow::bail!(
                "cached blockhash is older than {} seconds",
                MAX_BLOCKHASH_AGE.as_secs()
            );
        }
        Ok(cached.hash)
    }
}

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
    max_event_age_ms: u64,
    wait_tx_confirmed: bool,
    wait_for_all_submits: bool,
    assume_prepared_atas: bool,
    buy_mode: BuyMode,
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
    let trade_client = Arc::new(create_trade_client(&settings).await?);
    let blockhash_cache = BlockhashCache::start(trade_client.infrastructure.rpc.clone()).await?;
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

    let stream_client = ShredStreamClient::new_with_config(
        settings.shredstream_endpoint.clone(),
        ShredStreamConfig::low_latency(),
    )
    .await
    .map_err(|err| anyhow::anyhow!("failed to create ShredStream client: {}", err))?;
    let queue = stream_client
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
                    if select_trade_event(
                        &e,
                        settings.target_mint,
                        settings.require_created_buy,
                        settings.max_event_age_ms,
                    ) {
                        println!(
                            "matched event: mint={}, sig={}, slot={}, ix={}",
                            e.mint, e.metadata.signature, e.metadata.slot, e.ix_name
                        );
                        execute_buy_hold_sell(
                            settings.clone(),
                            e,
                            trade_client.clone(),
                            blockhash_cache.clone(),
                        )
                        .await?;
                        stream_client.stop().await;
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

        let settings = Self {
            payer,
            rpc_url,
            shredstream_endpoint,
            target_mint,
            require_created_buy: env_bool("REQUIRE_CREATED_BUY", true),
            buy_lamports: (buy_sol * LAMPORTS_PER_SOL as f64).round() as u64,
            buy_slippage_bps: env_u64("BUY_SLIPPAGE_BPS", 300)?,
            sell_slippage_bps: env_u64("SELL_SLIPPAGE_BPS", 500)?,
            hold_seconds: env_u64("HOLD_SECONDS", 3)?,
            max_event_age_ms: env_u64("MAX_EVENT_AGE_MS", 1_000)?,
            wait_tx_confirmed: env_bool("WAIT_TX_CONFIRMED", true),
            wait_for_all_submits: env_bool("WAIT_FOR_ALL_SUBMITS", false),
            assume_prepared_atas: env_bool("ASSUME_PREPARED_ATAS", false),
            buy_mode: match env_string("BUY_MODE")
                .unwrap_or_else(|| "with_max_input".to_string())
                .to_ascii_lowercase()
                .as_str()
            {
                "with_max_input" => BuyMode::WithMaxInput,
                "exact_input" => BuyMode::ExactInput,
                value => {
                    anyhow::bail!("BUY_MODE must be with_max_input or exact_input, got {value}")
                }
            },
        };
        validate_settings(&settings)?;
        Ok(settings)
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
    max_event_age_ms: u64,
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
    if !is_event_fresh(event.metadata.grpc_recv_us, now_micros(), max_event_age_ms) {
        eprintln!(
            "ignoring stale event for mint {} (MAX_EVENT_AGE_MS={})",
            event.mint, max_event_age_ms
        );
        return false;
    }
    true
}

fn is_event_fresh(recv_us: i64, now_us: i64, max_age_ms: u64) -> bool {
    recv_us > 0
        && now_us >= recv_us
        && now_us.saturating_sub(recv_us) <= (max_age_ms as i64).saturating_mul(1_000)
}

async fn execute_buy_hold_sell(
    settings: Settings,
    event: PumpFunTradeEvent,
    client: Arc<SolanaTrade>,
    blockhash_cache: BlockhashCache,
) -> Result<()> {
    let gas_fee_strategy = default_gas_fee_strategy();

    validate_trade_event_for_buy(&event)?;

    let balance_before = client
        .get_payer_token_balance_with_program(&event.mint, &event.token_program)
        .await
        .context("failed to read token balance before buy")?;

    let recent_blockhash = blockhash_cache.latest()?;
    let account_policy = if settings.assume_prepared_atas {
        AccountPolicy::HotPathMinimal
    } else {
        AccountPolicy::Auto
    };
    let buy_params = SimpleBuyParams::new(
        DexType::PumpFun,
        TradeTokenType::SOL,
        event.mint,
        settings.buy_mode.amount(settings.buy_lamports),
        DexParamEnum::PumpFun(build_buy_params(&client, &event).await?),
        recent_blockhash,
        gas_fee_strategy.clone(),
    )
    .slippage_basis_points(settings.buy_slippage_bps)
    .account_policy(account_policy)
    .wait_tx_confirmed(settings.wait_tx_confirmed)
    .wait_for_all_submits(settings.wait_for_all_submits)
    .grpc_recv_us(event.metadata.grpc_recv_us);

    println!("buying {}...", event.mint);
    let (ok, sigs, err, _) = client.buy_simple(buy_params).await?;
    if !ok {
        let message = err
            .map(|e| e.to_string())
            .unwrap_or_else(|| "unknown buy error".to_string());
        anyhow::bail!("buy failed: {} | sigs: {:?}", message, sigs);
    }
    println!("buy submitted: {:?}", sigs);

    println!("holding {} seconds before sell...", settings.hold_seconds);
    tokio::time::sleep(Duration::from_secs(settings.hold_seconds)).await;

    let balance_after = client
        .get_payer_token_balance_with_program(&event.mint, &event.token_program)
        .await
        .context("failed to read token balance before sell")?;
    let position_amount = balance_after.checked_sub(balance_before).ok_or_else(|| {
        anyhow::anyhow!(
            "token balance decreased from {} to {}; refusing to sell existing holdings",
            balance_before,
            balance_after
        )
    })?;
    if position_amount == 0 {
        anyhow::bail!("confirmed buy did not increase the token balance; refusing to sell");
    }

    let sell_recent_blockhash = blockhash_cache.latest()?;
    let sell_params = SimpleSellParams::new(
        DexType::PumpFun,
        TradeTokenType::SOL,
        event.mint,
        SellAmount::ExactInput(position_amount),
        DexParamEnum::PumpFun(build_fresh_sell_params(&client, &event).await?),
        sell_recent_blockhash,
        gas_fee_strategy,
    )
    .slippage_basis_points(settings.sell_slippage_bps)
    .account_policy(AccountPolicy::Auto)
    .wait_tx_confirmed(settings.wait_tx_confirmed)
    .wait_for_all_submits(settings.wait_for_all_submits);

    println!("selling this run's {} tokens...", position_amount);
    let (ok, sigs, err, _) = client.sell_simple(sell_params).await?;
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
        return Ok(PumpFunParams::from_dev_trade_with_quote_mint(
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
            event.quote_mint,
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

fn validate_settings(settings: &Settings) -> Result<()> {
    if settings.buy_lamports == 0 {
        anyhow::bail!("BUY_SOL_AMOUNT must produce at least one lamport");
    }
    if settings.buy_slippage_bps >= 10_000 || settings.sell_slippage_bps >= 10_000 {
        anyhow::bail!("BUY_SLIPPAGE_BPS and SELL_SLIPPAGE_BPS must be below 10000");
    }
    if settings.max_event_age_ms == 0 || settings.max_event_age_ms > i64::MAX as u64 / 1_000 {
        anyhow::bail!(
            "MAX_EVENT_AGE_MS must be between 1 and {}",
            i64::MAX / 1_000
        );
    }
    if !settings.wait_tx_confirmed {
        anyhow::bail!("WAIT_TX_CONFIRMED must be true when automatic selling is enabled");
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

#[cfg(test)]
mod tests {
    use super::is_event_fresh;

    #[test]
    fn event_freshness_rejects_missing_future_and_stale_timestamps() {
        assert!(!is_event_fresh(0, 1_000_000, 100));
        assert!(!is_event_fresh(1_000_001, 1_000_000, 100));
        assert!(!is_event_fresh(899_999, 1_000_000, 100));
        assert!(is_event_fresh(900_000, 1_000_000, 100));
    }
}
