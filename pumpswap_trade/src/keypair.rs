//! 从私钥字符串加载 Keypair

use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;

/// 从私钥字符串加载 Keypair
/// 支持格式：
/// 1) 标准 64 字节数组 JSON（例如 [1,2,3,...64]）
/// 2) base58 私钥
pub fn load_keypair_from_string(private_key_str: &str) -> anyhow::Result<Keypair> {
    let trimmed = private_key_str.trim();
    if trimmed.is_empty() {
        anyhow::bail!("私钥为空");
    }

    // 尝试作为标准 64 字节数组 JSON 解析
    if let Ok(bytes) = serde_json::from_str::<Vec<u8>>(trimmed) {
        if bytes.len() == 64 {
            if let Ok(k) = Keypair::try_from(bytes.as_slice()) {
                println!("  钱包地址: {}（已按标准密钥 JSON 加载）", k.pubkey());
                return Ok(k);
            }
        }
    }

    // 尝试作为 base58 私钥解析
    if trimmed.len() > 50 && !trimmed.starts_with('[') && !trimmed.starts_with('{') {
        let k = Keypair::from_base58_string(trimmed);
        println!("  钱包地址: {}（已按 base58 私钥加载）", k.pubkey());
        return Ok(k);
    }

    anyhow::bail!(
        "私钥格式无法识别。支持格式：\n\
         1) 标准 64 字节数组 JSON（例如 [1,2,3,...64]）\n\
         2) base58 私钥"
    )
}
