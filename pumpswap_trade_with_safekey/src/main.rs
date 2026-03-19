//! PumpSwap 买→等 30 秒→卖→等 30 秒，最多重复 3 次；可配置多 SWQoS 同时发交易。
//!
//! 无需 gRPC / sol-parser-sdk，启动后：每轮买入 → 休息 30s → 卖出 → 休息 30s，共 3 轮。
//!
//! 环境变量：
//! - MINT：代币 mint 地址（必填，或第一个命令行参数）
//! - SOLANA_RPC_URL：RPC 地址（可被 config 覆盖）
//! - BUY_SOL_AMOUNT：买入用 SOL 数量（浮点，如 0.01，默认 0.01）
//! - KEYSTORE_PASSWORD：keystore.json 密码（可选，不设则运行时交互输入）
//! - CONFIG_FILE / APP_ENV：配置路径或环境(dev/prod)，见 config/dev|prod/solana.yaml
//! - NONCE_ACCOUNT：多 SWQoS 时 durable nonce 可由此指定；若已在 solana.yaml 的 nonce_config 中配置则无需设置
//!
//! 钱包：优先使用 config 中 keystore_path 指向的 keystore.json，运行时需输入密码（或 KEYSTORE_PASSWORD）。
//! 未配置 keystore_path 时可退回到 KEYPAIR_BASE58。

mod config;
mod client;
mod keypair;
mod run;
mod swqos;

use sol_trade_sdk::{
    common::nonce_cache::fetch_nonce_info,
    find_pool_by_mint,
    swqos::{SwqosConfig, SwqosType},
    trading::factory::DexType,
};
use solana_sdk::native_token::LAMPORTS_PER_SOL;
use solana_sdk::signature::Keypair;
use solana_sdk::pubkey::Pubkey;
use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;

const DEFAULT_BUY_SOL_AMOUNT: f64 = 0.01;
const CLIENT_INIT_TIMEOUT_SECS: u64 = 90;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    load_dotenv();

    let mint = std::env::args()
        .nth(1)
        .or_else(|| std::env::var("MINT").ok())
        .expect("用法: pumpswap_buy_sell_swqos <MINT> 或设置 MINT 环境变量");

    let config_path = resolve_config_path();
    let trading_path = {
        let config_dir = Path::new(&config_path).parent().unwrap_or_else(|| Path::new("."));
        let path = config_dir.join("trading.yaml");
        path.to_str().unwrap_or("trading.yaml").to_string()
    };

    let (mut rpc_url, swqos_configs, keystore_path, nonce_config, trading_config) =
        load_config(&config_path, &trading_path)?;

    if let Ok(url) = std::env::var("SOLANA_RPC_URL") {
        let url = url.trim();
        if !url.is_empty() {
            rpc_url = url.to_string();
        }
    }
    if rpc_url.contains("api.mainnet-beta.solana.com") {
        eprintln!("⚠️  当前使用公网 RPC，易卡顿。请在 .env 中设置 SOLANA_RPC_URL 或运行前 export SOLANA_RPC_URL=你的RPC地址");
    }

    let buy_sol_amount: f64 = trading_config
        .as_ref()
        .map(|t| t.buy_sol_amount)
        .or_else(|| std::env::var("BUY_SOL_AMOUNT").ok().and_then(|s| s.trim().parse().ok()))
        .unwrap_or(DEFAULT_BUY_SOL_AMOUNT);
    let sol_lamports = (buy_sol_amount * LAMPORTS_PER_SOL as f64).round() as u64;

    let payer = load_payer(&keystore_path)?;

    println!(
        "=== PumpSwap 买→休息 {} 秒→卖→休息 {} 秒，共 {} 轮（多 SWQoS 同时发交易）===",
        run::rest_secs(),
        run::rest_secs(),
        run::rounds()
    );
    println!("  MINT: {}", mint);
    println!(
        "  SOL: {} lamports ({:.4} SOL)",
        sol_lamports,
        sol_lamports as f64 / LAMPORTS_PER_SOL as f64
    );
    println!("  RPC: {}", rpc_url);

    let swqos_types: Vec<SwqosType> = swqos_configs.iter().map(SwqosConfig::swqos_type).collect();
    let swqos_count = swqos_configs.len();

    println!("\n[0] 初始化交易客户端（不在此步创建 WSOL ATA，买入时按需创建）...");
    let client = match tokio::time::timeout(
        std::time::Duration::from_secs(CLIENT_INIT_TIMEOUT_SECS),
        client::create_client(&rpc_url, payer, swqos_configs),
    )
    .await
    {
        Ok(Ok(c)) => c,
        Ok(Err(e)) => anyhow::bail!("创建交易客户端失败: {}", e),
        Err(_) => anyhow::bail!(
            "初始化交易客户端超时（{} 秒）。请检查：1) RPC 地址 {} 是否可达 2) 网络或防火墙 3) 换用更快的 RPC",
            CLIENT_INIT_TIMEOUT_SECS,
            rpc_url
        ),
    };
    println!("[0] ✓ 交易客户端创建完成");

    let env_nonce = std::env::var("NONCE_ACCOUNT")
        .ok()
        .and_then(|s| {
            let t = s.trim();
            if t.is_empty() {
                None
            } else {
                Some(t.to_string())
            }
        });

    let (durable_nonce_buy, durable_nonce_sell) = if swqos_count > 1 {
        let buy_str = nonce_config
            .as_ref()
            .and_then(|n| n.buy_nonce_accounts.first().cloned())
            .or(env_nonce.clone())
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "已配置 {} 个 SWQoS，多 SWQoS 同时发交易需设置 durable nonce。请在 solana.yaml 的 nonce_config.buy_nonce_accounts 中配置，或在 .env 中设置 NONCE_ACCOUNT，或仅保留 1 个 SWQoS 配置。",
                    swqos_count
                )
            })?;
        let sell_str = nonce_config
            .as_ref()
            .and_then(|n| n.sell_nonce_accounts.first().cloned())
            .or_else(|| env_nonce.clone())
            .unwrap_or_else(|| buy_str.clone());
        let buy_pubkey = Pubkey::from_str(&buy_str)?;
        let buy_nonce = fetch_nonce_info(&client.infrastructure.rpc, buy_pubkey)
            .await
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "无法获取 nonce 账户 {} 的状态，请确认地址正确且已初始化",
                    buy_str
                )
            })?;
        println!("[0a] ✓ 买入 durable nonce（账户 {}）", buy_str);
        let sell_nonce = if sell_str == buy_str {
            buy_nonce.clone()
        } else {
            let sell_pubkey = Pubkey::from_str(&sell_str)?;
            let s = fetch_nonce_info(&client.infrastructure.rpc, sell_pubkey)
                .await
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "无法获取 nonce 账户 {} 的状态，请确认地址正确且已初始化",
                        sell_str
                    )
                })?;
            println!("[0a] ✓ 卖出 durable nonce（账户 {}）", sell_str);
            s
        };
        (Some(buy_nonce), Some(sell_nonce))
    } else {
        (None, None)
    };

    let mint_pubkey = Pubkey::from_str(&mint)?;

    println!("\n[1] 查找 PumpSwap 池...");
    let pool = find_pool_by_mint(&client.infrastructure.rpc, &mint_pubkey, DexType::PumpSwap)
        .await
        .map_err(|e| anyhow::anyhow!("查找池失败（错误信息中含诊断，请查看）: {}", e))?;
    println!("[1] ✓ 池地址: {}", pool);

    let buy_slippage_bps = trading_config.as_ref().map(|t| t.buy_slippage_bps).unwrap_or(500);
    let sell_slippage_bps = trading_config.as_ref().map(|t| t.sell_slippage_bps).unwrap_or(9980);

    run::run_pumpswap_loop(
        &client,
        mint_pubkey,
        pool,
        trading_config.as_ref(),
        &swqos_types,
        durable_nonce_buy,
        durable_nonce_sell,
        sol_lamports,
        buy_slippage_bps,
        sell_slippage_bps,
    )
    .await
}

fn load_dotenv() {
    if dotenvy::dotenv().is_err() {
        if let Ok(exe) = std::env::current_exe() {
            if let Some(p) = exe.parent() {
                for rel in [
                    "examples/pumpswap_buy_sell_swqos/.env",
                    "../../examples/pumpswap_buy_sell_swqos/.env",
                    "../.env",
                    "../../.env",
                ] {
                    let p = p.join(rel);
                    if p.exists() {
                        let _ = dotenvy::from_path(p);
                        break;
                    }
                }
            }
        }
    }
}

fn resolve_config_path() -> String {
    std::env::var("CONFIG_FILE").unwrap_or_else(|_| {
        let env = std::env::var("APP_ENV").unwrap_or_else(|_| "dev".to_string());
        let dir = if env == "prod" || env == "production" {
            "prod"
        } else {
            "dev"
        };
        format!("config/{}/solana.yaml", dir)
    })
}

fn load_config(
    config_path: &str,
    trading_path: &str,
) -> anyhow::Result<(
    String,
    Vec<SwqosConfig>,
    String,
    Option<config::NonceConfig>,
    Option<config::TradingConfig>,
)> {
    if !Path::new(config_path).exists() {
        let rpc_url = std::env::var("SOLANA_RPC_URL")
            .unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string());
        let configs = vec![SwqosConfig::Default(rpc_url.clone())];
        let keystore = std::env::var("KEYSTORE_PATH").unwrap_or_default();
        return Ok((rpc_url, configs, keystore, None, None));
    }

    let cfg = config::load_solana_config(Path::new(config_path))?;
    let configs = swqos::build_swqos_configs(&cfg);
    println!("  已加载 {} 个 SWQoS 配置: {}", config_path, configs.len());

    let trading_cfg = config::load_trading_config(Path::new(trading_path));
    if trading_cfg.is_some() {
        println!("  已加载交易与 Gas 费配置: {}", trading_path);
    }

    Ok((
        cfg.rpc_url,
        configs,
        cfg.keystore_path,
        cfg.nonce_config,
        trading_cfg,
    ))
}

fn load_payer(keystore_path: &str) -> anyhow::Result<Arc<Keypair>> {
    if !keystore_path.trim().is_empty() {
        println!("  使用 keystore: {}", keystore_path.trim());
        Ok(Arc::new(keypair::load_keypair_from_keystore(keystore_path.trim())?))
    } else {
        let keypair_b58 = std::env::var("KEYPAIR_BASE58").map_err(|_| {
            anyhow::anyhow!(
                "未配置 keystore_path 且未设置 KEYPAIR_BASE58，请在 config 中设置 keystore_path 或设置环境变量 KEYPAIR_BASE58"
            )
        })?;
        Ok(Arc::new(Keypair::from_base58_string(&keypair_b58)))
    }
}
