//! 交易客户端创建
//!
//! 使用方式 C（组合使用）：SWQOS 使用末尾 N 核 + 专用发送线程绑定同组核心，减少与主线程/默认 worker 争用。

use sol_trade_sdk::{
    common::{AnyResult, TradeConfig},
    recommended_sender_thread_core_indices,
    swqos::SwqosConfig,
    SolanaTrade,
};
use solana_commitment_config::CommitmentConfig;
use solana_sdk::signature::Keypair;
use std::sync::Arc;

pub async fn create_client(
    rpc_url: &str,
    payer: Arc<Keypair>,
    swqos_configs: Vec<SwqosConfig>,
) -> AnyResult<SolanaTrade> {
    let commitment = CommitmentConfig::confirmed();
    let swqos_count = swqos_configs.len();
    println!("  [create_client] 构建 TradeConfig（方式 C：末尾核 + 专用发送线程）...");
    let trade_config = TradeConfig::builder(rpc_url.to_string(), swqos_configs, commitment)
        .create_wsol_ata_on_startup(false)
        .use_seed_optimize(true)
        .swqos_cores_from_end(true)
        .build();
    println!("  [create_client] 调用 SolanaTrade::new...");
    let client = SolanaTrade::new(payer, trade_config).await;
    let client = match recommended_sender_thread_core_indices(swqos_count) {
        Some(indices) => {
            println!("  [create_client] 启用专用发送线程并绑定末尾 {} 核", indices.len());
            client.with_dedicated_sender_threads(Some(indices))
        }
        None => client,
    };
    println!("  [create_client] SolanaTrade::new 返回");
    Ok(client)
}
