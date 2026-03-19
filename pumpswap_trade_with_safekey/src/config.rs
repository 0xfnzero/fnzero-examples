//! 配置结构体与加载：solana.yaml、trading.yaml

use std::path::Path;

#[derive(serde::Deserialize, Default)]
pub struct NonceConfig {
    #[serde(default)]
    pub buy_nonce_accounts: Vec<String>,
    #[serde(default)]
    pub sell_nonce_accounts: Vec<String>,
}

#[derive(serde::Deserialize)]
pub struct SolanaConfig {
    pub rpc_url: String,
    #[serde(default)]
    pub keystore_path: String,
    #[serde(default)]
    pub nonce_config: Option<NonceConfig>,
    pub swqos: SwqosConfigSettings,
}

#[derive(serde::Deserialize)]
pub struct SwqosConfigSettings {
    pub region: String,
    pub enabled_providers: Vec<SwqosProviderConfig>,
}

#[derive(serde::Deserialize)]
pub struct SwqosProviderConfig {
    pub provider: String,
    pub api_token: Option<String>,
    pub enabled: bool,
    #[serde(default)]
    pub transport: Option<String>,
}

#[derive(serde::Deserialize, Default)]
pub struct TradingConfig {
    #[serde(default = "default_buy_sol_amount")]
    pub buy_sol_amount: f64,
    #[serde(default = "default_buy_slippage_bps")]
    pub buy_slippage_bps: u64,
    #[serde(default = "default_sell_slippage_bps")]
    pub sell_slippage_bps: u64,
    #[serde(default)]
    pub enable_high_low_fee: bool,
    #[serde(default)]
    pub gas_fee: GasFeeConfig,
}

fn default_buy_sol_amount() -> f64 { 0.01 }
fn default_buy_slippage_bps() -> u64 { 500 }
fn default_sell_slippage_bps() -> u64 { 9980 }

#[derive(serde::Deserialize)]
pub struct GasFeeConfig {
    #[serde(default = "default_cu_limit")]
    pub cu_limit: u32,
    #[serde(default = "default_buy_cu_limit")]
    pub global_buy_cu_limit: u32,
    #[serde(default = "default_sell_cu_limit")]
    pub global_sell_cu_limit: u32,
    #[serde(default = "default_buy_cu_price")]
    pub global_buy_cu_price: u64,
    #[serde(default = "default_sell_cu_price")]
    pub global_sell_cu_price: u64,
    #[serde(default = "default_global_buy_tip")]
    pub global_buy_tip: f64,
    #[serde(default = "default_global_sell_tip")]
    pub global_sell_tip: f64,
    #[serde(default = "default_low_cu_price")]
    pub low_tip_high_cu_price: u64,
    #[serde(default = "default_low_buy_tip")]
    pub low_buy_tip: f64,
    #[serde(default = "default_low_sell_tip")]
    pub low_sell_tip: f64,
    #[serde(default = "default_high_cu_price")]
    pub high_tip_low_cu_price: u64,
    #[serde(default = "default_high_buy_tip")]
    pub high_buy_tip: f64,
    #[serde(default = "default_high_sell_tip")]
    pub high_sell_tip: f64,
}

impl Default for GasFeeConfig {
    fn default() -> Self {
        Self {
            cu_limit: 200000,
            global_buy_cu_limit: 98467,
            global_sell_cu_limit: 93334,
            global_buy_cu_price: 12868833,
            global_sell_cu_price: 120000,
            global_buy_tip: 0.00115,
            global_sell_tip: 0.00015,
            low_tip_high_cu_price: 400000,
            low_buy_tip: 0.003,
            low_sell_tip: 0.001,
            high_tip_low_cu_price: 180000,
            high_buy_tip: 0.003,
            high_sell_tip: 0.001,
        }
    }
}

fn default_cu_limit() -> u32 { 200000 }
fn default_buy_cu_limit() -> u32 { 98467 }
fn default_sell_cu_limit() -> u32 { 93334 }
fn default_buy_cu_price() -> u64 { 12868833 }
fn default_sell_cu_price() -> u64 { 120000 }
fn default_global_buy_tip() -> f64 { 0.00115 }
fn default_global_sell_tip() -> f64 { 0.00015 }
fn default_low_cu_price() -> u64 { 400000 }
fn default_low_buy_tip() -> f64 { 0.003 }
fn default_low_sell_tip() -> f64 { 0.001 }
fn default_high_cu_price() -> u64 { 180000 }
fn default_high_buy_tip() -> f64 { 0.003 }
fn default_high_sell_tip() -> f64 { 0.001 }

/// 从路径加载 solana.yaml
pub fn load_solana_config(config_path: &Path) -> anyhow::Result<SolanaConfig> {
    let yaml = std::fs::read_to_string(config_path)?;
    let cfg = serde_yaml::from_str(&yaml)?;
    Ok(cfg)
}

/// 从路径加载 trading.yaml，文件不存在或解析失败时返回 None
pub fn load_trading_config(trading_path: &Path) -> Option<TradingConfig> {
    if !trading_path.exists() {
        return None;
    }
    let yaml = std::fs::read_to_string(trading_path).ok()?;
    serde_yaml::from_str::<TradingConfig>(&yaml).ok()
}
