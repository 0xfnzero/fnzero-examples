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
    <a href="#chinese">中文</a> |
    <a href="#english">English</a> |
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
cd fnzero-examples/pumpswap_buy_sell_swqos
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

# Keystore 密码（可选，不设置则交互输入）
KEYSTORE_PASSWORD=

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

#### 创建 keystore.json

**方式 1：使用 sol-safekey（推荐）**

```bash
# 安装 sol-safekey
cargo install sol-safekey

# 生成加密 keystore
sol-safekey new keystore.json

# 按提示输入密码（建议 10-20 位字符）
```

**方式 2：转换现有密钥对**

```bash
# 从 base58 私钥转换
echo "your_base58_private_key" > key.txt
sol-safekey import key.txt keystore.json

# 从 JSON 数组（64 字节）转换
cat keypair.json | sol-safekey import - keystore.json
```

#### 创建必需账户

**1. WSOL ATA（Wrapped SOL 关联代币账户）**

WSOL ATA 会在首次买入操作时自动创建，无需手动设置。

**2. Durable Nonce 账户**

使用 2 个及以上 SWQoS 服务时，需要创建 Durable Nonce 账户以实现交易重放保护。

```bash
# 安装 Solana CLI
sh -c "$(curl -sSfL https://release.solana.com/stable/install)"

# 生成 durable nonce 密钥对
solana-keygen new --outfile nonce-keypair.json

# 创建 nonce 账户
solana create-nonce-account nonce-keypair.json

# 获取 nonce 地址
solana-keygen pubkey nonce-keypair.json
```

将 nonce 地址添加到配置文件：

```yaml
# config/dev/solana.yaml 或 config/prod/solana.yaml
nonce_config:
  buy_nonce_accounts:
    - "YOUR_NONCE_PUBKEY_HERE"
  sell_nonce_accounts:
    - "YOUR_NONCE_PUBKEY_HERE"
```

**注意**：可以为买入和卖出使用不同的 nonce 账户，也可以使用同一个。

### 配置文件

配置文件组织在 `config/` 目录中：

```
config/
├── dev/
│   ├── solana.yaml      # RPC、keystore、SWQoS、nonce 配置
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

| 服务 | 必需参数 | 官网 |
|---------|-----------|-------|
| Astralane | API Key | https://astralane.io |
| BlockRazor | API Key | https://blockrazor.io |
| Jito | UUID | https://jito.wtf |
| NextBlock | API Key | - |
| Bloxroute | API Key | https://www.bloxroute.com |
| ZeroSlot | API Key | https://zeroslot.com |
| Temporal | API Key | https://temporal.cloud |
| FlashBlock | API Key | - |
| Node1 | API Key | - |

申请 API 密钥：https://fnzero.dev/swqos

---

## 🚀 使用方法

### 使用命令行参数运行

```bash
./pumpswap_buy_sell_swqos <MINT_ADDRESS>
```

### 使用环境变量运行

```bash
MINT=<MINT_ADDRESS> ./pumpswap_buy_sell_swqos
```

### 使用 .env 文件运行

```bash
# 在 .env 中设置 MINT 或作为参数传入
./pumpswap_buy_sell_swqos
```

### 使用不同环境运行

```bash
APP_ENV=prod ./pumpswap_buy_sell_swqos <MINT_ADDRESS>
```

---

## 🔧 构建

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
├── x86_64-unknown-linux-gnu/release/pumpswap_buy_sell_swqos
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
