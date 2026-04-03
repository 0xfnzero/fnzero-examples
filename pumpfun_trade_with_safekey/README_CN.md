<div align="center">
    <h1>🎯 PumpFun 内盘自动交易示例（加密钱包）</h1>
    <h3><em>Keystore / KEYPAIR_BASE58 + 多 SWQoS</em></h3>
</div>

<p align="center">
    <strong>与 <a href="../pumpfun_trade/README_CN.md">pumpfun_trade</a> 交易逻辑相同，钱包改为 <code>sol-safekey</code> 加密文件或备用 <code>KEYPAIR_BASE58</code>。</strong>
</p>

<p align="center">
    <a href="README_CN.md">中文</a> |
    <a href="README.md">English</a> |
    <a href="../README_CN.md">仓库总览</a>
</p>

---

<p align="center"><a href="../README_CN.md">← 返回仓库总览</a></p>

---

## 适用场景

- 代币须在 **PumpFun 内盘**（未毕业）；已毕业请用 `pumpswap_trade_with_safekey`。
- 在 `config/*/solana.yaml` 中配置 **`keystore_path`**（如 `./keystore.json`），运行时输入密码或设置 **`KEYSTORE_PASSWORD`**。
- 若未配置 `keystore_path`，可设置环境变量 **`KEYPAIR_BASE58`**（仅应急，安全性低于 keystore）。

## 生成 keystore

在仓库中执行（路径按实际放置调整）：

```bash
cd sol-safekey
cargo run --release -- export <私钥或助记词> ../pumpfun_trade_with_safekey/keystore.json
```

然后在 `pumpfun_trade_with_safekey/config/dev/solana.yaml` 中写入 `keystore_path: "./keystore.json"`。

更完整的 keystore、SWQoS、nonce 说明见 [pumpswap_trade_with_safekey/README_CN.md](../pumpswap_trade_with_safekey/README_CN.md)（协议为 PumpSwap，配置结构相同）。

## 运行前准备

1. 按仓库根目录 [README_CN.md](../README_CN.md) 单独克隆 **sol-safekey** 并生成 `keystore.json`（本仓库不包含该目录）。
2. 在本目录执行：

```bash
cp .env.example .env
cp config/dev/solana.yaml.example config/dev/solana.yaml
cp config/dev/trading.yaml.example config/dev/trading.yaml
```

3. 编辑 `solana.yaml`：`keystore_path`、SWQoS、nonce 等。`.env` / `yaml` **勿提交 Git**。

## 运行与构建

```bash
cd pumpfun_trade_with_safekey
./run.sh <MINT_ADDRESS>
```

```bash
cargo build --release
# 二进制：build-cache/release/pumpfun_trade_with_safekey
./build-linux-release.sh   # Linux 部署包
```

## 功能摘要

- 默认 **1 轮**：买 → 等约 30 秒 → 卖光该 mint 余额（见 `src/run.rs` 中 `ROUNDS`）。
- **≥2 个 SWQoS** 需配置 durable nonce；YAML 中空字符串占位不生效时可依赖 **`NONCE_ACCOUNT`**。
- 卖出前重新 `from_mint_by_rpc`，与 `pumpfun_trade` 一致。

---

## 📄 许可证

MIT License

## 💬 联系

https://fnzero.dev/ · https://github.com/0xfnzero/fnzero-examples
