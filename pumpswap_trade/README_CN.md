<div align="center">
    <h1>🔄 PumpSwap 自动交易示例</h1>
    <h3><em>在 PumpSwap 上使用多个 SWQoS 实现自动化的买入-等待-卖出循环交易策略</em></h3>
</div>

<p align="center">
    <strong>Rust 示例项目，演示在 PumpSwap 上的自动化交易策略：买入 → 等待 30 秒 → 卖出 → 等待 30 秒，共 3 轮。支持多个 SWQoS 服务同时提交交易。</strong>
</p>

<p align="center">
    <a href="https://github.com/0xfnzero/fnzero-examples">
        <img src="https://img.shields.io/github/stars/0xfnzero/fnzero-examples?style=social" alt="GitHub stars">
    </a>
    <a href="https://github.com/0xfnzero/fnzero-examples/network">
        <img src="https://img.shields.io/github/forks/0xfnzero/fnzero-examples?style=social" alt="GitHub forks">
    </a>
</p>

<p align="center">
    <img src="https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white" alt="Rust">
    <img src="https://img.shields.io/badge/Solana-9945FF?style=for-the-badge&logo=solana&logoColor=white" alt="Solana">
    <img src="https://img.shields.io/badge/PumpSwap-4B8BBE?style=for-the-badge" alt="PumpSwap">
</p>

<p align="center">
    <a href="README_CN.md">中文</a> |
    <a href="README.md">English</a> |
    <a href="https://fnzero.dev/">官网</a> |
    <a href="https://t.me/fnzero_group">Telegram</a> |
    <a href="https://discord.gg/vuazbGkqQE">Discord</a>
</p>

---

## 📋 目录

- [✨ 功能特性](#-功能特性)
- [📦 安装](#-安装)
- [🛠️ 配置](#️-配置)
  - [环境变量](#环境变量)
  - [钱包设置](#钱包设置)
  - [配置文件](#配置文件)
  - [SWQoS 服务](#swqos-服务)
- [🚀 使用方法](#-使用方法)
- [🔧 构建](#-构建)
- [📄 许可证](#-许可证)
- [💬 联系方式](#-联系方式)

---

## ✨ 功能特性

1. **自动化交易循环**：买入 → 等待 30 秒 → 卖出 → 等待 30 秒，共 3 轮
2. **多 SWQoS 支持**：向多个 MEV 保护服务并发提交交易
3. **Durable Nonce**：多 SWQoS 场景下的交易重放保护
4. **灵活配置**：支持 YAML 配置文件和环境变量
5. **环境变量优先级**：环境变量可覆盖配置文件中的敏感信息
6. **跨平台构建**：支持 Linux 部署的构建脚本

---

## 📦 安装

### 克隆仓库

```bash
git clone https://github.com/0xfnzero/fnzero-examples.git
cd fnzero-examples/pumpswap_trade
```

### 依赖项

确保已安装 Rust 和 Cargo：
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

---

## 🛠️ 配置

### 环境变量

根据 `.env.example` 创建 `.env` 文件：

```bash
# 环境：dev 或 prod
APP_ENV=prod

# 代币 mint 地址（也可作为命令行参数传入）
MINT=

# 钱包私钥（支持 base58 或标准 64 字节数组 JSON 格式）
PRIVATE_KEY=

# RPC 地址
SOLANA_RPC_URL=http://your-rpc-endpoint.com

# 买入 SOL 数量
BUY_SOL_AMOUNT=0.01

# SWQoS 区域
SWQOS_REGION=Frankfurt

# SWQoS 提供商 token
SWQOS_ASTRALANE_TOKEN=
SWQOS_BLOCKRAZOR_TOKEN=
SWQOS_JITO_TOKEN=
SWQOS_ZEROSLOT_TOKEN=
# ... 等等

# Durable nonce 账户（使用 2 个及以上 SWQoS 时必需）
NONCE_ACCOUNT=
```

### 钱包设置

本项目支持直接配置私钥。你可以通过以下两种方式配置钱包：

#### 方式一：使用 .env 文件（推荐）

1. 复制 `.env.example` 为 `.env`
2. 将你的私钥添加到 `PRIVATE_KEY` 变量

```bash
cp .env.example .env
```

编辑 `.env`：
```bash
PRIVATE_KEY=你的私钥
```

**私钥格式支持：**
- **Base58 格式**：标准的 Solana 私钥字符串（例如来自 `solana-keygen new`）
- **64 字节数组 JSON**：某些钱包导出的 `[1,2,3,...64]` 格式

#### 方式二：使用配置文件

你也可以直接在配置文件中设置私钥：

```yaml
# config/dev/solana.yaml 或 config/prod/solana.yaml
private_key: "你的私钥"
```

**安全提示：**
- 永远不要将 `.env` 文件提交到版本控制
- 在生产环境中使用环境变量以获得更好的安全性
- 妥善保管你的私钥，切勿泄露

### 配置文件

配置文件组织在 `config/` 目录中：

```
config/
├── dev/
│   ├── solana.yaml      # RPC、私钥、SWQoS、nonce 配置
│   └── trading.yaml    # 交易参数、gas 费设置
└── prod/
    ├── solana.yaml
    └── trading.yaml
```

**环境变量覆盖**：
- 配置文件中的所有敏感值都可以被环境变量覆盖
- `.env` 中的值优先级高于配置文件
- 适合将敏感信息排除在版本控制之外

### SWQoS 服务

支持以下 SWQoS 服务：

| 服务 | 必需参数 |
|---------|-----------|
| Astralane | API Key |
| BlockRazor | API Key |
| Jito | UUID |
| NextBlock | API Key |
| Bloxroute | API Key |
| ZeroSlot | API Key |
| Temporal | API Key |
| FlashBlock | API Key |
| Node1 | API Key |

申请 API 密钥：https://fnzero.dev/swqos

---

## 🚀 使用方法

### 快速开始

直接运行脚本：

```bash
./run.sh <MINT_ADDRESS>
```

脚本会：
- 从 `.env` 和 `config/dev/solana.yaml` 加载配置
- 以 release 模式构建并运行交易机器人
- 通过 `APP_ENV` 变量支持开发/生产环境

### 使用命令行参数运行

```bash
./pumpswap_trade <MINT_ADDRESS>
```

### 使用环境变量运行

```bash
MINT=<MINT_ADDRESS> ./pumpswap_trade
```

### 使用 .env 文件运行

```bash
# 在 .env 中设置 MINT 或作为参数传入
./pumpswap_trade
```

### 使用不同环境运行

```bash
APP_ENV=prod ./pumpswap_trade <MINT_ADDRESS>
```

---

## 🔧 构建

### 本地开发构建

```bash
cargo build --release
```

二进制文件将生成在 `target/release/pumpswap_trade`

### 构建 Linux Release

```bash
./build-linux-release.sh
```

该脚本会：
1. 交叉编译为 Linux 二进制（x86_64-unknown-linux-gnu）
2. 打包二进制文件和配置到 `linux-release/deploy.tar.gz`

输出：
```
linux-release/
├── x86_64-unknown-linux-gnu/release/pumpswap_trade
└── deploy.tar.gz
```

---

## 📄 许可证

MIT License

---

## 💬 联系方式

- 官方网站：https://fnzero.dev/
- 项目仓库：https://github.com/0xfnzero/fnzero-examples
- Telegram 群组：https://t.me/fnzero_group
- Discord：https://discord.gg/vuazbGkqQE
