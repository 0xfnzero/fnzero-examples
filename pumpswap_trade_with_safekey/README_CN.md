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
cd fnzero-examples/pumpswap_trade_with_safekey
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

#### 完整的钱包设置教程（推荐）

本教程将带你使用 **sol-safekey** 完成所有钱包设置，包括创建 keystore.json、解锁钱包、创建 WSOL ATA 和创建 durable nonce 账户。

##### 步骤 1：安装 sol-safekey

**从源码编译安装（推荐）：**

```bash
# 1. 克隆或进入 sol-safekey 项目目录
cd /path/to/sol-safekey

# 2. 使用 full feature 编译并安装
cargo install --path . --features="full"

# 3. 验证安装
sol-safekey --version
```

**从 crates.io 安装：**

```bash
cargo install sol-safekey --features="full"
```

##### 步骤 2：启动交互式菜单

```bash
sol-safekey start
```

你将看到语言选择界面：

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  Sol-SafeKey - Solana 密钥管理工具
  Solana Security Key Management Tool
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Please select language | 请选择语言:

  1. 中文
  2. English

Select language (1-2): _
```

输入 `2` 选择中文。



##### 步骤 3：创建 keystore.json 钱包

选择语言后，进入主菜单：

```

==================================================
  Sol-SafeKey - Solana 密钥管理工具
==================================================

核心功能 (只需3个操作):

  1.  创建明文私钥
  2.  创建加密私钥(bot)
  3.  解密私钥

  🔒 钱包状态: 未解锁
  U.  解锁钱包（用于Solana操作）

  高级安全功能:
  4.  设置 2FA 认证
  5.  生成三因子钱包
  6.  解锁三因子钱包

  Solana 链上操作:
  7.  查询 SOL 余额
  8.  转账 SOL
  9.  创建 WSOL ATA
  10.  包装 SOL → WSOL
  11.  解包 WSOL → SOL
  12.  关闭 WSOL ATA
  13.  转账 SPL 代币
  14.  创建 Nonce 账户
  15.  Pump.fun 卖出代币
  16.  PumpSwap 卖出代币
  17.  Pump.fun 返现（查看与领取）
  18.  PumpSwap 返现（查看与领取）

  0.  退出

选择操作 (0-18/U): _
```

**重要提示**：由于你还没有创建钱包，首先需要**解锁钱包**或**创建新钱包**。

**创建新钱包的步骤：**
1. 选择 `1. 创建明文私钥`
2. 按提示生成密钥对
3. 按提示保存到文件

**加密现有私钥的步骤：**
1. 选择 `2. 创建加密私钥`
2. 输入或粘贴现有私钥
3. 按提示输入密码
4. 按提示输入密码确认
5. 选择保存文件名（默认 keystore.json）

**解锁钱包的步骤：**
1. 选择 `U` 解锁钱包
2. 按提示输入 keystore 文件路径（默认 keystore.json）
3. 按提示输入密码


密码强度验证通过！✅

正在加密私钥...
✅ 钱包已保存: keystore.json

📝 重要提醒:
   • 密码请妥善保管，丢失无法恢复！
   • keystore.json 文件包含了加密后的私钥
   • 建议将 keystore.json 备份到多个安全位置
```

##### 步骤 4：解锁钱包

返回主菜单，输入 `U` 解锁钱包：

```
请选择操作 (1-6/U/Q): U
```

**操作步骤：**
1. 系统会提示输入 keystore 文件路径（默认为 `keystore.json`）
2. 直接回车使用默认路径
3. 输入之前设置的密码
4. 解锁成功后，钱包将保存在会话中

**示例输出：**

```
  解锁钱包
Keystore 文件路径 [keystore.json]:

请输入密码: ********

✅ 钱包解锁成功！
📍 当前钱包: 7xKm...9xW3
```

##### 步骤 5：创建 WSOL ATA 账号

解锁钱包后，选择 `4` 进入 Solana 操作菜单：

```
请选择操作 (1-6/U/Q): 4

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
              Solana 操作菜单 | Solana Operations
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  1.  查询余额 (查询余额)
  2.  创建 WSOL ATA (创建 WSOL ATA)
  3.  SOL ⟷ WSOL (打包 SOL)
  4.  WSOL ⟷ SOL (解包 WSOL)
  5.  创建 Nonce 账户 (创建 Nonce 账户)
  B.  返回 (返回)

请选择操作 (1-5/B): _
```

**操作步骤：**
1. 输入 `2` 并回车
2. 系统会自动创建 WSOL Associated Token Account
3. 等待交易确认

**示例输出：**

```
📝 创建 WSOL ATA

🚀 正在创建 WSOL Associated Token Account...
✅ WSOL ATA 创建成功！

📍 ATA 地址: 7xKm...9xW3
📊 Token Mint: So11111111111111111111111111111111111111111112

Signature: 5xKm...9xW3
Explorer: https://solscan.io/tx/5xKm...9xW3
```

##### 步骤 6：创建 Durable Nonce 账号

返回 Solana 操作菜单，输入 `5` 创建 Nonce 账号：

```
请选择操作 (1-5/B): 5

🔑 创建 Nonce 账户

ℹ️  A nonce account will be created for durable transactions
ℹ️  将创建一个用于持久交易的 Nonce 账户
```

**操作步骤：**
1. 系统会自动创建一个新的 nonce 账号
2. nonce 账号用于确保交易的幂等性和防止重放攻击
3. 等待交易确认
4. **重要**：记录创建的 nonce 账户地址，用于配置文件

**示例输出：**

```
🚀 正在创建 Nonce 账户...

✅ Nonce 账户创建和初始化成功！
   📍 地址: 5xKm...7xW3
   🔐 Nonce值: 1234abcd...efgh5678

💡 请保存此 Nonce 账户地址以供将来使用！
```

##### 步骤 7：配置 Nonce 账户

将上面创建的 nonce 账户地址添加到配置文件：

```yaml
# config/dev/solana.yaml 或 config/prod/solana.yaml
nonce_config:
  buy_nonce_accounts:
    - "5xKm...7xW3"  # 使用实际创建的地址
  sell_nonce_accounts:
    - "5xKm...7xW3"  # 可以使用同一个，或创建不同的
```

**注意**：
- WSOL ATA 会在首次买入操作时自动创建，无需手动设置
- 使用 2 个及以上 SWQoS 服务时，必须配置 durable nonce 账户
- 可以为买入和卖出使用不同的 nonce 账户，也可以使用同一个

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

### 快速开始

直接运行脚本：

```bash
./run.sh
```

脚本会：
- 从 `.env` 和 `config/dev/solana.yaml` 加载配置
- 提示输入 keystore 密码（除非设置了 `KEYSTORE_PASSWORD`）
- 以 release 模式构建并运行交易机器人
- 通过 `APP_ENV` 变量支持开发/生产环境

### 使用命令行参数运行

```bash
./pumpswap_trade_with_safekey <MINT_ADDRESS>
```

### 使用环境变量运行

```bash
MINT=<MINT_ADDRESS> ./pumpswap_trade_with_safekey
```

### 使用 .env 文件运行

```bash
# 在 .env 中设置 MINT 或作为参数传入
./pumpswap_trade_with_safekey
```

### 使用不同环境运行

```bash
APP_ENV=prod ./pumpswap_trade_with_safekey <MINT_ADDRESS>
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
├── x86_64-unknown-linux-gnu/release/pumpswap_trade_with_safekey
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
