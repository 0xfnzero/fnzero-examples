<div align="center">
    <h1>🚀 FnZero Solana 示例项目</h1>
    <h3><em>Solana DEX 交易与开发的实用示例和工具集</em></h3>
</div>

<p align="center">
    <strong>Rust 示例、SDK 和工具集合，用于构建高性能 Solana 交易机器人。包括交易示例、DEX 集成 SDK、交易解析、密钥管理和实时流式处理。</strong>
</p>

<p align="center">
    <a href="https://github.com/0xfnzero/fnzero-examples">
        <img src="https://img.shields.io/github/stars/0xfnzero/fnzero-examples?style=social" alt="GitHub stars">
    </a>
    <a href="https://github.com/0xfnzero/fnzero-examples/network">
        <img src="https://img.shields.io/github/forks/0xfnzero/fnzero-examples?style=social" alt="GitHub forks">
    </a>
    <a href="https://github.com/0xfnzero/fnzero-examples/blob/main/LICENSE">
        <img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License">
    </a>
</p>

<p align="center">
    <img src="https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white" alt="Rust">
    <img src="https://img.shields.io/badge/Solana-9945FF?style=for-the-badge&logo=solana&logoColor=white" alt="Solana">
    <img src="https://img.shields.io/badge/DEX-4B8BBE?style=for-the-badge&logo=bitcoin&logoColor=white" alt="DEX Trading">
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

- [✨ 特性](#-特性)
- [📁 项目结构](#-项目结构)
- [🛠️ 示例](#️-示例)
- [📦 SDK 与工具](#-sdk-与工具)
- [🚀 快速开始](#-快速开始)
- [📄 许可证](#-许可证)
- [💬 联系方式](#-联系方式)

---

## ✨ 特性

- **交易示例**: 即用型 PumpSwap 交易机器人示例，支持自动化买入/卖出策略
- **多 SWQoS 支持**: 通过多个 MEV 保护服务并发提交交易
- **全面的 SDK**: 用于交易、解析、密钥管理和流式处理的模块化 SDK
- **实时流式处理**: 基于 gRPC 的交易流式处理和低延迟事件解析
- **安全密钥管理**: 支持密码保护的加密密钥库
- **生产就绪**: 优化的构建配置，支持跨平台（Linux、macOS）

## 📁 项目结构

```
fnzero-examples/
├── pumpswap_trade/              # PumpSwap 交易示例（直接私钥）
├── pumpswap_trade_with_safekey/ # PumpSwap 交易示例（加密密钥库）
├── sol-trade-sdk/              # 统一 DEX 交易 SDK
├── sol-parser-sdk/             # 交易解析 SDK（gRPC 流式处理）
├── sol-safekey/                # 加密密钥管理库
└── solana-streamer/            # Solana 交易流式处理工具
```

---

## 🛠️ 示例

### 交易示例

| 示例 | 描述 | 运行命令 | 源代码 |
|---------|-------------|-------------|-------------|
| **PumpSwap 交易** | PumpSwap 自动化买→等→卖循环交易，可配置轮次和休息间隔 | `./run.sh` | [pumpswap_trade](./pumpswap_trade/) |
| **PumpSwap 交易（加密）** | 同上，但使用带密码保护的加密密钥库文件 | `./run.sh` | [pumpswap_trade_with_safekey](./pumpswap_trade_with_safekey/) |

### 示例特性

两个交易示例均包含：

- ✅ **自动化交易循环**: 买入 → 等待 30 秒 → 卖出 → 等待 30 秒，重复 3 轮（可配置）
- ✅ **多 SWQoS**: 通过多个 MEV 保护服务并发提交交易
- ✅ **灵活配置**: 基于 YAML 的开发/生产环境配置
- ✅ **Durable Nonce 支持**: 多 SWQoS 场景下的交易重放保护
- ✅ **Gas 费策略**: 可配置的优先费用和计算单元价格
- ✅ **滑点保护**: 可自定义买入/卖出滑点设置
- ✅ **环境变量**: 通过 `.env` 文件覆盖配置
- ✅ **跨平台构建**: 支持 Linux 和 macOS，优化的发布配置

### 配置说明

每个示例支持：

- **环境变量**: `.env` 文件用于敏感配置
- **YAML 配置**: `config/dev/solana.yaml` 和 `config/prod/solana.yaml`
- **示例模板**: `.yaml.example` 文件作为配置模板

#### 支持的 SWQoS 服务

- Astralane（HTTP & QUIC）
- BlockRazor
- Bloxroute
- FlashBlock
- Jito
- NextBlock
- Node1
- Speedlanding
- Soyas
- Stellium
- Temporal
- ZeroSlot
- Default（RPC）

---

## 📦 SDK 与工具

### sol-trade-sdk

全面的 Solana DEX 交易 Rust SDK，提供多协议统一接口。

**特性：**
- 支持 PumpFun、PumpSwap、Bonk、Raydium CPMM、Raydium AMM V4、Meteora DAMM V2
- 多个 MEV 保护服务（Jito、BlockRazor、Astralane 等）
- 中间件系统，支持自定义指令修改
- 多钱包场景的共享基础设施
- 地址查找表（ALT）支持
- Durable Nonce 管理
- Gas 费策略优化

**文档:** [sol-trade-sdk/README_CN.md](./sol-trade-sdk/README_CN.md)

### sol-parser-sdk

交易解析 SDK，支持 gRPC 流式处理和实时事件处理。

**特性：**
- 解析 PumpFun、PumpSwap、Raydium 等交易
- 基于 gRPC 的流式处理，低延迟事件处理
- 事件过滤和转换
- 交易事件提取
- 账户填充器优化账户查询

**文档:** [sol-parser-sdk/README_CN.md](./sol-parser-sdk/README_CN.md)

### sol-safekey

加密密钥管理库，用于安全存储 Solana 密钥对。

**特性：**
- 使用密码保护加密/解密 Solana 密钥对
- 基于 JSON 的密钥库格式
- Base58 私钥支持
- CLI 密钥管理工具
- 与交易示例集成

**文档:** [sol-safekey/README_CN.md](./sol-safekey/README_CN.md)

### solana-streamer

Solana 交易流式处理和实时数据处理工具。

**特性：**
- Shred 流订阅
- 交易流式处理
- 事件处理管道
- 性能优化的解析

**文档:** [solana-streamer/README_CN.md](./solana-streamer/README_CN.md)

---

## 🚀 快速开始

### 前置要求

- Rust 1.70+ 和 Cargo
- Solana CLI（可选，用于钱包管理）
- Solana RPC 端点（mainnet-beta 或 devnet）

### 克隆仓库

```bash
git clone https://github.com/0xfnzero/fnzero-examples.git
cd fnzero-examples
```

### 运行交易示例

**方式 1: 使用直接私钥**

```bash
cd pumpswap_trade

# 配置设置
cp .env.example .env
# 编辑 .env，填入你的私钥和 RPC URL

# 配置 SWQoS 服务
cp config/dev/solana.yaml.example config/dev/solana.yaml
# 编辑 config/dev/solana.yaml，填入你的 API tokens

# 运行交易机器人
cargo run --release -- <TOKEN_MINT_ADDRESS>
```

**方式 2: 使用加密密钥库**

```bash
cd pumpswap_trade_with_safekey

# 生成加密密钥库
cargo run --bin sol-safekey -- export <你的私钥或助记词> ./keystore.json

# 配置设置
cp .env.example .env
cp config/dev/solana.yaml.example config/dev/solana.yaml

# 运行交易机器人（会提示输入密码）
cargo run --release -- <TOKEN_MINT_ADDRESS>
```

### 配置详情

#### 环境变量（.env）

```bash
# 环境：dev 或 prod
APP_ENV=dev

# 要交易的代币 mint 地址
MINT=your_token_mint_address

# 买入 SOL 数量
BUY_SOL_AMOUNT=0.01

# RPC URL
SOLANA_RPC_URL=https://your-rpc-endpoint.com

# 密钥库密码（加密密钥使用）
KEYSTORE_PASSWORD=your_password

# Durable nonce 账户（多 SWQoS 时必需）
NONCE_ACCOUNT=your_nonce_account_address
```

#### SWQoS 配置（config/dev/solana.yaml）

```yaml
swqos:
  region: "Frankfurt"  # 或 NewYork、Tokyo 等
  enabled_providers:
    - provider: "Astralane"
      api_token: "your_token"
      enabled: true
    - provider: "BlockRazor"
      api_token: "your_token"
      enabled: true
    # 根据需要添加更多提供商
```

### 生产环境构建

```bash
# 优化的发布构建
cargo build --release

# 交叉编译到 Linux（从 macOS）
cargo build --release --target x86_64-unknown-linux-gnu

# 运行编译的二进制文件
./target/release/pumpswap_trade_with_safekey <TOKEN_MINT_ADDRESS>
```

---

## 📄 许可证

MIT License

详见 [LICENSE](./LICENSE)。

---

## 💬 联系方式

- **官方网站**: https://fnzero.dev/
- **项目仓库**: https://github.com/0xfnzero/fnzero-examples
- **Telegram 群组**: https://t.me/fnzero_group
- **Discord 服务器**: https://discord.gg/vuazbGkqQE

---

## ⚠️ 重要提示

1. **安全**: 永远不要将私钥或 API tokens 提交到版本控制
2. **测试**: 在主网使用前请在 devnet 上充分测试
3. **风险**: 加密货币交易存在重大风险
4. **合规**: 确保遵守当地法律法规
5. **RPC 限制**: 监控 RPC 使用以避免速率限制
6. **MEV 服务**: 正确配置 MEV 保护服务的 API tokens

## 🤝 贡献

欢迎贡献！请随时提交 Pull Request。

## 📚 额外资源

- [Solana 官方文档](https://docs.solana.com/)
- [Solana Cookbook](https://solanacookbook.com/)
- [Rust 文档](https://www.rust-lang.org/docs.html)

---

<div align="center">
    <strong>由 FnZero 团队用 ❤️ 构建</strong>
</div>
