<div align="center">
    <h1>🎯 PumpFun 内盘自动交易示例</h1>
    <h3><em>在 PumpFun bonding curve 上使用多 SWQoS：买入 → 等待 → 卖出</em></h3>
</div>

<p align="center">
    <strong>适用于尚未毕业到 PumpSwap、仍在 PumpFun bonding curve 上的代币。私钥通过 <code>PRIVATE_KEY</code> 或 <code>config/*/solana.yaml</code> 的 <code>private_key</code> 配置。</strong>
</p>

<p align="center">
    <a href="https://github.com/0xfnzero/fnzero-examples">
        <img src="https://img.shields.io/github/stars/0xfnzero/fnzero-examples?style=social" alt="GitHub stars">
    </a>
</p>

<p align="center">
    <img src="https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white" alt="Rust">
    <img src="https://img.shields.io/badge/Solana-9945FF?style=for-the-badge&logo=solana&logoColor=white" alt="Solana">
    <img src="https://img.shields.io/badge/PumpFun-14F195?style=for-the-badge" alt="PumpFun">
</p>

<p align="center">
    <a href="README_CN.md">中文</a> |
    <a href="README.md">English</a> |
    <a href="../README_CN.md">仓库总览</a>
</p>

> **提示**：代币已 **毕业到 PumpSwap 外盘** 时，请使用 **`pumpswap_trade`** / **`pumpswap_trade_with_safekey`**。

---

<p align="center"><a href="../README_CN.md">← 返回仓库总览</a></p>

---

## 📋 目录

- 何时使用本示例
- 功能特性 · 安装 · 配置 · 运行 · 构建
- 与 PumpSwap 示例的区别

---

## 何时使用本示例

| 使用 `pumpfun_trade` | 请改用 `pumpswap_trade` |
|----------------------|-------------------------|
| 代币仍在 **PumpFun 内盘**（bonding curve） | 代币已 **毕业** 到 PumpSwap 外盘 AMM |
| `PumpFunParams::from_mint_by_rpc` 能成功 | 需要按 mint 查找 **池地址** 走 PumpSwap |

若误对内盘示例传入已毕业 mint，程序在「解析 bonding curve」步骤会失败。

---

## 功能特性

1. **协议**：`DexType::PumpFun`，链上参数通过 `PumpFunParams::from_mint_by_rpc` 获取；**卖出前再次拉取**，避免 `creator_vault` 等字段过期。
2. **流程**：买入 → 等待约 30 秒 → 按**钱包该 mint 的全部余额**卖出；默认 **1 轮**（修改 `src/run.rs` 中 `ROUNDS`、`REST_SECS`）。
3. **多 SWQoS**、**Durable Nonce**（≥2 个 SWQoS 时必填有效 nonce）、**trading.yaml** 滑点与 Gas 配置——与 `pumpswap_trade` 一致。
4. **Nonce 占位**：`solana.yaml` 里 `buy_nonce_accounts: [""]` 等空串会被忽略，可仅用环境变量 `NONCE_ACCOUNT`。

---

## 安装

```bash
git clone https://github.com/0xfnzero/fnzero-examples.git
cd fnzero-examples/pumpfun_trade
```

需安装 Rust / Cargo：[https://rustup.rs](https://rustup.rs)

---

## 配置

- 复制 `.env.example` → `.env`，填写 `PRIVATE_KEY`、`SOLANA_RPC_URL` 等。
- 复制 `config/dev/solana.yaml.example` → `config/dev/solana.yaml`，启用 SWQoS 并填写 token。
- 交易参数见 `config/dev/trading.yaml`（如 `buy_sol_amount`、滑点、Gas）。

钱包与 SWQoS 详细说明可参考 [pumpswap_trade/README_CN.md](../pumpswap_trade/README_CN.md)（仅协议与 mint 要求不同）。

---

## 运行

```bash
./run.sh <MINT_ADDRESS>
# 或
cargo run --release -- <MINT_ADDRESS>
```

环境变量 `APP_ENV=dev|prod` 决定读取 `config/dev` 或 `config/prod`。

---

## 构建

本目录 `.cargo/config.toml` 将产物输出到 **`build-cache/release/`**。

```bash
cargo build --release
./build-cache/release/pumpfun_trade <MINT_ADDRESS>
```

Linux 部署包：

```bash
./build-linux-release.sh
```

---

## 与 PumpSwap 示例的区别

| 项目 | `pumpfun_trade` | `pumpswap_trade` |
|------|-----------------|------------------|
| DEX | PumpFun 内盘 | PumpSwap 外盘 |
| 池子 | 无独立 pool；由 mint 推导 bonding curve | `find_pool_by_mint` 查池 |
| Mint 要求 | 未毕业内盘币 | 已在外盘有池 |

加密钱包版本见 [pumpfun_trade_with_safekey/README_CN.md](../pumpfun_trade_with_safekey/README_CN.md)。

---

## 📄 许可证

MIT License

---

## 💬 联系方式

- 官网：https://fnzero.dev/
- 仓库：https://github.com/0xfnzero/fnzero-examples
