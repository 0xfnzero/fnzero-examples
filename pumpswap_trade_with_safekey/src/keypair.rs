//! 从 keystore 加载 Keypair

use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;
use std::path::Path;

/// 从 keystore 加载 Keypair：
/// 1) 先尝试 sol-safekey 加密格式（需密码）；
/// 2) 若解密失败且文件为标准 Solana 密钥 JSON（64 字节数组），则直接使用（无需密码）。
pub fn load_keypair_from_keystore(keystore_path: &str) -> anyhow::Result<Keypair> {
    if keystore_path.is_empty() {
        anyhow::bail!("keystore_path 为空");
    }
    let path = Path::new(keystore_path);
    if !path.exists() {
        anyhow::bail!(
            "keystore 文件不存在: {}\n  请将 keystore.json 放到该路径，或使用 sol-safekey 生成",
            keystore_path
        );
    }

    let content = std::fs::read_to_string(path)
        .map_err(|e| anyhow::anyhow!("读取 keystore 失败: {}", e))?;

    if content.contains("encrypted_private_key") {
        let keypair = match std::env::var("KEYSTORE_PASSWORD") {
            Ok(p) if !p.trim().is_empty() => {
                println!("  使用环境变量 KEYSTORE_PASSWORD 解锁 keystore");
                let pwd = p.trim().to_string();
                sol_safekey::KeyManager::keypair_from_encrypted_json(&content, &pwd)
                    .map_err(|e| anyhow::anyhow!("密码错误或钱包文件损坏: {}", e))?
            }
            _ => {
                sol_safekey::bot_helper::ensure_wallet_ready(keystore_path)
                    .map_err(|e| anyhow::anyhow!("密码错误或钱包文件损坏: {}", e))?
            }
        };
        println!("  钱包地址: {}", keypair.pubkey());
        return Ok(keypair);
    }

    if let Ok(bytes) = serde_json::from_str::<Vec<u8>>(&content) {
        if bytes.len() == 64 {
            if let Ok(k) = Keypair::try_from(bytes.as_slice()) {
                println!("  钱包地址: {}（已按标准密钥 JSON 加载，未加密）", k.pubkey());
                return Ok(k);
            }
        }
    }

    let line = content.lines().next().map(str::trim).unwrap_or("");
    if !line.is_empty() && line.len() > 50 && !line.starts_with('[') && !line.starts_with('{') {
        let k = Keypair::from_base58_string(line);
        println!("  钱包地址: {}（已按 base58 私钥加载）", k.pubkey());
        return Ok(k);
    }

    anyhow::bail!(
        "keystore 无法识别。支持：1) sol-safekey 加密 JSON（需正确密码 10–20 字符） 2) 标准 64 字节数组 JSON 3) 单行 base58 私钥。\
         \n若密码正确仍报 Invalid UTF-8，请用 sol-safekey 重新导出该钱包，或改用上述 2/3 格式。"
    )
}
