//! PumpFun 内盘买→等 30 秒→卖→等 30 秒，只执行一次；可配置多 SWQoS 同时发交易。
//!
//! 针对仍在 bonding curve 上的代币。钱包：keystore + 密码（或 KEYSTORE_PASSWORD）。
//!
//! 环境变量：
//! - MINT：代币 mint 地址（必填，或第一个命令行参数）
//! - SOLANA_RPC_URL、BUY_SOL_AMOUNT、CONFIG_FILE、APP_ENV、NONCE_ACCOUNT 等同 pumpswap_trade_with_safekey
//! - **APP_ENV**：未设置时默认为 `dev`，读取 `config/dev/solana.yaml`；生产配置（如已启用 Speedlanding）请 **`APP_ENV=prod`** 或 **`CONFIG_FILE=/…/config/prod/solana.yaml`**
//! - **SKIP_TRADING** / **PAUSE_TRADING**：设为 `1`、`true`、`yes` 时仅执行初始化与链上探测（含 bonding curve、nonce、run 内 ATA 等），**不发送买入/卖出交易**（用于修复 SWQoS 等期间避免实盘）
//! - KEYSTORE_PASSWORD：keystore.json 密码（可选，不设则运行时交互输入）

mod config;
mod client;
mod keypair;
mod run;
mod swqos;

use sol_trade_sdk::{
    common::nonce_cache::fetch_nonce_info,
    swqos::{SwqosConfig, SwqosType},
    trading::core::params::PumpFunParams,
};
use solana_sdk::native_token::LAMPORTS_PER_SOL;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;
use std::path::{Path, PathBuf};
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
        .expect("用法: pumpfun_trade_with_safekey <MINT> 或设置 MINT 环境变量");

    let config_path_raw = resolve_config_path();
    let config_path = absolutize_config_path(&config_path_raw);
    let trading_path = config_path
        .parent()
        .map(|d| d.join("trading.yaml"))
        .unwrap_or_else(|| PathBuf::from("trading.yaml"));

    let (mut rpc_url, swqos_configs, keystore_path, nonce_config, trading_config) =
        load_config(&config_path, &trading_path)?;
    let keystore_path = resolve_keystore_path(&config_path, &keystore_path);

    println!("  配置文件: {}", config_path.display());
    println!(
        "  APP_ENV: {}",
        std::env::var("APP_ENV").unwrap_or_else(|_| "dev (默认)".to_string())
    );
    if swqos_configs.len() == 1 && matches!(swqos_configs[0], SwqosConfig::Default(_)) {
        eprintln!(
            "  提示: 当前 SWQoS 仅为 Default（RPC）。若你在 config/prod/solana.yaml 中启用了 Speedlanding 等，请使用生产配置: APP_ENV=prod 或 CONFIG_FILE=<项目根>/config/prod/solana.yaml"
        );
    }

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
        "=== PumpFun 内盘 买→休息 {} 秒→卖→休息 {} 秒，共 {} 轮（多 SWQoS 同时发交易）===",
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

    let skip_trading = env_truthy("SKIP_TRADING") || env_truthy("PAUSE_TRADING");
    if skip_trading {
        println!("  SKIP_TRADING: 已启用 → 将跳过买入/卖出，仅完成初始化与链上探测");
    }

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
            .and_then(|n| first_nonempty_account(&n.buy_nonce_accounts))
            .or(env_nonce.clone())
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "已配置 {} 个 SWQoS，多 SWQoS 同时发交易需设置 durable nonce。请在 solana.yaml 的 nonce_config.buy_nonce_accounts 中配置，或在 .env 中设置 NONCE_ACCOUNT，或仅保留 1 个 SWQoS 配置。",
                    swqos_count
                )
            })?;
        let sell_str = nonce_config
            .as_ref()
            .and_then(|n| first_nonempty_account(&n.sell_nonce_accounts))
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

    println!("\n[1] 解析 PumpFun bonding curve（内盘）...");
    let _probe = PumpFunParams::from_mint_by_rpc(&client.infrastructure.rpc, &mint_pubkey)
        .await
        .map_err(|e| {
            anyhow::anyhow!(
                "无法加载 PumpFun 参数（代币可能已毕业到 PumpSwap 或 mint 无效）: {}",
                e
            )
        })?;
    println!("[1] ✓ bonding curve 就绪（DexType::PumpFun）");

    let buy_slippage_bps = trading_config.as_ref().map(|t| t.buy_slippage_bps).unwrap_or(500);
    let sell_slippage_bps = trading_config.as_ref().map(|t| t.sell_slippage_bps).unwrap_or(9980);

    run::run_pumpfun_loop(
        &client,
        mint_pubkey,
        trading_config.as_ref(),
        &swqos_types,
        durable_nonce_buy,
        durable_nonce_sell,
        sol_lamports,
        buy_slippage_bps,
        sell_slippage_bps,
        skip_trading,
    )
    .await
}

fn env_truthy(name: &str) -> bool {
    std::env::var(name)
        .ok()
        .map(|s| {
            matches!(
                s.trim().to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on"
            )
        })
        .unwrap_or(false)
}

fn first_nonempty_account(list: &[String]) -> Option<String> {
    list.iter()
        .map(|s| s.trim())
        .find(|s| !s.is_empty())
        .map(|s| s.to_string())
}

fn load_dotenv() {
    if dotenvy::dotenv().is_err() {
        if let Ok(exe) = std::env::current_exe() {
            if let Some(p) = exe.parent() {
                for rel in [
                    "examples/pumpfun_trade_with_safekey/.env",
                    "../../examples/pumpfun_trade_with_safekey/.env",
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

/// 将 `CONFIG_FILE` / 默认相对路径转为绝对路径（基于当前工作目录），避免依赖「未规范化的相对路径字符串」拼接导致 keystore 路径错位。
fn absolutize_config_path(config_file: &str) -> PathBuf {
    let p = Path::new(config_file);
    let joined = if p.is_absolute() {
        p.to_path_buf()
    } else {
        std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(p)
    };
    std::fs::canonicalize(&joined).unwrap_or(joined)
}

/// 非绝对路径时相对于已绝对化的 `solana.yaml` 路径所在目录解析 keystore。
fn resolve_keystore_path(config_file: &Path, keystore_from_yaml: &str) -> String {
    let s = keystore_from_yaml.trim();
    if s.is_empty() {
        return String::new();
    }
    let p = Path::new(s);
    if p.is_absolute() {
        return s.to_string();
    }
    let base = config_file.parent().unwrap_or_else(|| Path::new("."));
    base.join(p).to_string_lossy().into_owned()
}

fn load_config(
    config_path: &Path,
    trading_path: &Path,
) -> anyhow::Result<(
    String,
    Vec<SwqosConfig>,
    String,
    Option<config::NonceConfig>,
    Option<config::TradingConfig>,
)> {
    if !config_path.exists() {
        let rpc_url = std::env::var("SOLANA_RPC_URL")
            .unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string());
        let configs = vec![SwqosConfig::Default(rpc_url.clone())];
        let keystore = std::env::var("KEYSTORE_PATH").unwrap_or_default();
        return Ok((rpc_url, configs, keystore, None, None));
    }

    let cfg = config::load_solana_config(config_path)?;
    let configs = swqos::build_swqos_configs(&cfg);
    println!(
        "  已加载 {} 个 SWQoS 配置: {}",
        config_path.display(),
        configs.len()
    );

    let trading_cfg = config::load_trading_config(trading_path);
    if trading_cfg.is_some() {
        println!("  已加载交易与 Gas 费配置: {}", trading_path.display());
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
        let k = Keypair::from_base58_string(&keypair_b58);
        println!("  使用 KEYPAIR_BASE58，钱包地址: {}", k.pubkey());
        Ok(Arc::new(k))
    }
}
