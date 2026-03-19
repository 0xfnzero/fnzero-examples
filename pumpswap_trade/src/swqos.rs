//! SWQoS 配置构建与 Gas 费策略

use sol_trade_sdk::{
    common::GasFeeStrategy,
    swqos::{SwqosConfig, SwqosRegion, SwqosType, TradeType},
};

use crate::config::{SolanaConfig, TradingConfig};

pub fn parse_region(s: &str) -> SwqosRegion {
    match s {
        "NewYork" => SwqosRegion::NewYork,
        "Frankfurt" => SwqosRegion::Frankfurt,
        "Amsterdam" => SwqosRegion::Amsterdam,
        "SLC" => SwqosRegion::SLC,
        "Tokyo" => SwqosRegion::Tokyo,
        "London" => SwqosRegion::London,
        "LosAngeles" => SwqosRegion::LosAngeles,
        _ => SwqosRegion::NewYork,
    }
}

pub fn build_swqos_configs(cfg: &SolanaConfig) -> Vec<SwqosConfig> {
    let region = if let Ok(r) = std::env::var("SWQOS_REGION") {
        parse_region(&r)
    } else {
        parse_region(&cfg.swqos.region)
    };
    let rpc_url = &cfg.rpc_url;
    let mut out = Vec::new();
    for p in &cfg.swqos.enabled_providers {
        if !p.enabled {
            continue;
        }
        // 优先从环境变量获取 token，否则使用配置文件中的 token
        let token = get_provider_token(&p.provider)
            .or_else(|| p.api_token.as_ref().cloned())
            .unwrap_or_default();
        if let Some(c) = create_swqos_config(&p.provider, token, &region, rpc_url) {
            out.push(c);
        }
    }
    if out.is_empty() {
        eprintln!("⚠️  未启用任何 SWQoS provider。请在 config 中的 enabled_providers 中设置 enabled=true，或确保至少有一个 provider 启用");
        out.push(SwqosConfig::Default(rpc_url.clone()));
    }
    out
}

fn get_provider_token(provider_name: &str) -> Option<String> {
    let env_var = match provider_name {
        "Astralane" => std::env::var("SWQOS_ASTRALANE_TOKEN").ok(),
        "BlockRazor" => std::env::var("SWQOS_BLOCKRAZOR_TOKEN").ok(),
        "Jito" => std::env::var("SWQOS_JITO_TOKEN").ok(),
        "NextBlock" => std::env::var("SWQOS_NEXTBLOCK_TOKEN").ok(),
        "Bloxroute" => std::env::var("SWQOS_BLOXROUTE_TOKEN").ok(),
        "ZeroSlot" => std::env::var("SWQOS_ZEROSLOT_TOKEN").ok(),
        "Temporal" => std::env::var("SWQOS_TEMPORAL_TOKEN").ok(),
        "FlashBlock" => std::env::var("SWQOS_FLASHBLOCK_TOKEN").ok(),
        "Node1" => std::env::var("SWQOS_NODE1_TOKEN").ok(),
        "Stellium" => std::env::var("SWQOS_STELLIUM_TOKEN").ok(),
        "Speedlanding" => std::env::var("SWQOS_SPEEDLANDING_TOKEN").ok(),
        "Soyas" => std::env::var("SWQOS_SOYAS_TOKEN").ok(),
        _ => None,
    };
    env_var.filter(|s| !s.trim().is_empty())
}

pub fn create_swqos_config(
    provider_name: &str,
    api_token: String,
    region: &SwqosRegion,
    rpc_url: &str,
) -> Option<SwqosConfig> {
    let token = api_token;
    let reg = region.clone();
    match provider_name {
        "Jito" => Some(SwqosConfig::Jito(token, reg, None)),
        "NextBlock" => Some(SwqosConfig::NextBlock(token, reg, None)),
        "Bloxroute" => Some(SwqosConfig::Bloxroute(token, reg, None)),
        "ZeroSlot" => Some(SwqosConfig::ZeroSlot(token, reg, None)),
        "Temporal" => Some(SwqosConfig::Temporal(token, reg, None)),
        "FlashBlock" => Some(SwqosConfig::FlashBlock(token, reg, None)),
        "Node1" => Some(SwqosConfig::Node1(token, reg, None, None)),
        "BlockRazor" => Some(SwqosConfig::BlockRazor(token, reg, None, None)),
        "Astralane" => Some(SwqosConfig::Astralane(token, reg, None, None)),
        "Stellium" => Some(SwqosConfig::Stellium(token, reg, None)),
        "Lightspeed" => Some(SwqosConfig::Lightspeed(token, reg, None)),
        "Soyas" => Some(SwqosConfig::Soyas(token, reg, None)),
        "Speedlanding" => Some(SwqosConfig::Speedlanding(token, reg, None)),
        "Default" => Some(SwqosConfig::Default(rpc_url.to_string())),
        _ => Some(SwqosConfig::Default(rpc_url.to_string())),
    }
}

pub fn build_gas_fee_strategy(
    trading_config: Option<&TradingConfig>,
    swqos_types: &[SwqosType],
) -> GasFeeStrategy {
    let gas = GasFeeStrategy::new();
    if let Some(t) = trading_config {
        let g = &t.gas_fee;
        if t.enable_high_low_fee {
            gas.set_high_low_fee_strategies(
                swqos_types,
                TradeType::Buy,
                g.cu_limit,
                g.high_tip_low_cu_price,
                g.low_tip_high_cu_price,
                g.low_buy_tip,
                g.high_buy_tip,
            );
            gas.set_high_low_fee_strategies(
                swqos_types,
                TradeType::Sell,
                g.cu_limit,
                g.high_tip_low_cu_price,
                g.low_tip_high_cu_price,
                g.low_sell_tip,
                g.high_sell_tip,
            );
        } else {
            gas.set_global_fee_strategy(
                g.global_buy_cu_limit,
                g.global_sell_cu_limit,
                g.global_buy_cu_price,
                g.global_sell_cu_price,
                g.global_buy_tip,
                g.global_sell_tip,
            );
        }
    } else {
        gas.set_global_fee_strategy(150_000, 150_000, 500_000, 500_000, 0.001, 0.001);
    }
    gas
}
