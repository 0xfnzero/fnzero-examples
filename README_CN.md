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
- [📎 示例文档索引](#示例文档索引)
- [📦 SDK 与工具](#-sdk-与工具)
- [🚀 快速开始](#-快速开始)
- [📄 许可证](#-许可证)
- [💬 联系方式](#-联系方式)

---

## ✨ 特性

- **交易示例**: PumpSwap 外盘与 PumpFun 内盘交易机器人示例，支持自动化买入/卖出策略
- **多 SWQoS 支持**: 通过多个 MEV 保护服务并发提交交易
- **全面的 SDK**: 用于交易、解析、密钥管理和流式处理的模块化 SDK
- **实时流式处理**: 基于 gRPC 的交易流式处理和低延迟事件解析
- **安全密钥管理**: 支持密码保护的加密密钥库
- **生产就绪**: 优化的构建配置，支持跨平台（Linux、macOS）

## 📁 项目结构

```
fnzero-examples/
├── pumpswap_trade/              # PumpSwap 外盘交易示例（直接私钥）
├── pumpswap_trade_with_safekey/ # PumpSwap 外盘交易示例（加密密钥库）
├── pumpfun_trade/               # PumpFun 内盘交易示例（直接私钥）
├── pumpfun_trade_with_safekey/  # PumpFun 内盘交易示例（加密密钥库）
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
| **PumpSwap 交易** | PumpSwap 外盘自动化买→等→卖循环交易，可配置轮次和休息间隔 | `./run.sh` | [pumpswap_trade](./pumpswap_trade/) |
| **PumpSwap 交易（加密）** | 同上，但使用带密码保护的加密密钥库文件 | `./run.sh` | [pumpswap_trade_with_safekey](./pumpswap_trade_with_safekey/) |
| **PumpFun 交易** | PumpFun 内盘（bonding curve）买→等→卖，代币须未毕业到 PumpSwap | `./run.sh` | [pumpfun_trade](./pumpfun_trade/) |
| **PumpFun 交易（加密）** | 同上，keystore / `KEYPAIR_BASE58` | `./run.sh` | [pumpfun_trade_with_safekey](./pumpfun_trade_with_safekey/) |

### 如何选择示例

| 场景 | 推荐目录 |
|------|----------|
| 代币仍在 **PumpFun 内盘**（bonding curve），尚未毕业到 PumpSwap | `pumpfun_trade` 或 `pumpfun_trade_with_safekey` |
| 代币已在 **PumpSwap 外盘** AMM | `pumpswap_trade` 或 `pumpswap_trade_with_safekey` |
| 私钥通过 `PRIVATE_KEY` 或 `solana.yaml` 的 `private_key` | `pumpfun_trade` / `pumpswap_trade` |
| 加密 **keystore** + 密码（或备用 `KEYPAIR_BASE58`） | `pumpfun_trade_with_safekey` / `pumpswap_trade_with_safekey` |

### 示例特性（四个交易示例共通）

- ✅ **自动化流程**: 买入 → 等待约 30 秒 → 卖出；**默认执行 1 轮**（可在各示例 `src/run.rs` 中修改 `ROUNDS`、`REST_SECS`）
- ✅ **多 SWQoS**: 多个 MEV 保护通道并发提交
- ✅ **YAML + .env**: `config/dev|prod/solana.yaml`、`trading.yaml` 与环境变量
- ✅ **Durable Nonce**: 启用 **2 个及以上** SWQoS 时需在 `nonce_config` 或环境变量 `NONCE_ACCOUNT` 中配置有效 nonce 账户；YAML 中占位 `""` 会被忽略并回退到 `NONCE_ACCOUNT`
- ✅ **Gas / 滑点**: `trading.yaml` 中可配
- ✅ **默认不等待链上确认**（`wait_tx_confirmed: false`），买入后依赖固定等待时间再查余额；生产环境可按需改为等待确认或加长等待
- ⚠️ **卖出数量**: 每轮按钱包该 mint 的**全部**代币余额卖出（含买入前已有持仓）

### 示例文档索引

| 示例 | 中文 README | English |
|------|----------------|---------|
| PumpSwap（私钥） | [pumpswap_trade/README_CN.md](./pumpswap_trade/README_CN.md) | [pumpswap_trade/README.md](./pumpswap_trade/README.md) |
| PumpSwap（加密） | [pumpswap_trade_with_safekey/README_CN.md](./pumpswap_trade_with_safekey/README_CN.md) | [pumpswap_trade_with_safekey/README.md](./pumpswap_trade_with_safekey/README.md) |
| PumpFun（私钥） | [pumpfun_trade/README_CN.md](./pumpfun_trade/README_CN.md) | [pumpfun_trade/README.md](./pumpfun_trade/README.md) |
| PumpFun（加密） | [pumpfun_trade_with_safekey/README_CN.md](./pumpfun_trade_with_safekey/README_CN.md) | [pumpfun_trade_with_safekey/README.md](./pumpfun_trade_with_safekey/README.md) |

### 配置说明

每个示例支持：

- **环境变量**: `.env` 文件用于敏感配置
- **YAML 配置**: `config/dev/solana.yaml` 和 `config/prod/solana.yaml`
- **示例模板**: `.yaml.example` 文件作为配置模板

#### 支持的 SWQoS 服务

| 服务 | 传输协议 |
|---------|---------------------|
| **Astralane** | HTTP, QUIC ⚡ |
| **BlockRazor** | HTTP, gRPC |
| **Bloxroute** | HTTP |
| **FlashBlock** | HTTP |
| **Jito** | HTTP |
| **NextBlock** | HTTP |
| **Node1** | HTTP, QUIC ⚡ |
| **Soyas** | QUIC ⚡ |
| **Speedlanding** | QUIC ⚡ |
| **Stellium** | HTTP |
| **Temporal** | HTTP |
| **ZeroSlot** | HTTP |
| **Default** | RPC |

**API Key 申请**：通过官方网站申请 API 密钥：[https://fnzero.dev/swqos](https://fnzero.dev/swqos)

**注意**：⚡ = QUIC（Quick UDP Internet Connections）相比 HTTP/gRPC 提供更低的延迟。使用 QUIC 的服务（Astralane、Node1、Soyas、Speedlanding）通常提供最佳的交易提交性能。

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

**须在对应示例子目录下执行**（每个目录是独立 Cargo 包，仓库根目录无 workspace）。

**方式 1：私钥（PumpSwap 或 PumpFun 二选一）**

```bash
cd pumpswap_trade          # 外盘；若做内盘则改为 cd pumpfun_trade

cp .env.example .env
# 编辑 .env：PRIVATE_KEY、SOLANA_RPC_URL 等

cp config/dev/solana.yaml.example config/dev/solana.yaml
# 编辑 solana.yaml：启用 SWQoS、填写 token；多 SWQoS 时配置 nonce 或设 NONCE_ACCOUNT

./run.sh <TOKEN_MINT_ADDRESS>
# 或: cargo run --release -- <TOKEN_MINT_ADDRESS>
```

**方式 2：加密 keystore**

```bash
cd sol-safekey
cargo run --release -- export <你的私钥或助记词> ../pumpswap_trade_with_safekey/keystore.json
cd ../pumpswap_trade_with_safekey   # PumpFun 则改为 pumpfun_trade_with_safekey

cp .env.example .env
cp config/dev/solana.yaml.example config/dev/solana.yaml
# solana.yaml 中设置 keystore_path（如 ./keystore.json）

./run.sh <TOKEN_MINT_ADDRESS>
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

各示例通过 `.cargo/config.toml` 将产物输出到 **`build-cache/release/`**（而非默认的 `target/release/`）。

```bash
cd pumpswap_trade    # 或 pumpfun_trade、pumpswap_trade_with_safekey、pumpfun_trade_with_safekey

cargo build --release
./build-cache/release/pumpswap_trade <TOKEN_MINT_ADDRESS>   # 二进制名与目录名一致

# 从 macOS 交叉编译 Linux（需已安装 x86_64-unknown-linux-gnu 工具链）
./build-linux-release.sh   # 生成 linux-release/deploy.tar.gz
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
