<div align="center">
    <h1>🚀 FnZero Solana Examples</h1>
    <h3><em>Practical examples and tools for Solana DEX trading and development</em></h3>
</div>

<p align="center">
    <strong>A collection of Rust examples, SDKs, and tools for building high-performance Solana trading bots. Includes trading examples, SDKs for DEX integration, transaction parsing, key management, and real-time streaming.</strong>
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
    <a href="https://fnzero.dev/">Website</a> |
    <a href="https://t.me/fnzero_group">Telegram</a> |
    <a href="https://discord.gg/vuazbGkqQE">Discord</a>
</p>

---

## 📋 Table of Contents

- [✨ Features](#-features)
- [📁 Project Structure](#-project-structure)
- [🛠️ Examples](#️-examples)
- [📎 Example README index](#example-readme-index)
- [📦 SDKs & Tools](#-sdks--tools)
- [🚀 Quick Start](#-quick-start)
- [📄 License](#-license)
- [💬 Contact](#-contact)

---

## ✨ Features

- **Trading Examples**: Ready-to-use bots for PumpSwap (outer pool) and PumpFun (bonding curve) with automated buy/sell
- **Multiple SWQoS Support**: Concurrent transaction submission via multiple MEV protection services
- **Comprehensive SDKs**: Modular SDKs for trading, parsing, key management, and streaming
- **Real-time Streaming**: gRPC-based transaction streaming and parsing with low-latency event processing
- **Secure Key Management**: Encrypted keystore support with password protection
- **Production Ready**: Optimized builds with cross-platform support (Linux, macOS)

## 📁 Project Structure

```
fnzero-examples/
├── pumpswap_trade/              # PumpSwap (outer pool) trading example (direct private key)
├── pumpswap_trade_with_safekey/ # PumpSwap (outer pool) trading example (encrypted keystore)
├── pumpfun_trade/               # PumpFun (bonding curve) trading example (direct private key)
├── pumpfun_trade_with_safekey/  # PumpFun (bonding curve) trading example (encrypted keystore)
├── sol-trade-sdk/              # Unified DEX trading SDK
├── sol-parser-sdk/             # Transaction parsing SDK (gRPC streaming)
├── sol-safekey/                # Encrypted key management library
└── solana-streamer/            # Solana transaction streaming utilities
```

---

## 🛠️ Examples

### Trading Examples

| Example | Description | Run Command | Source Code |
|---------|-------------|-------------|-------------|
| **PumpSwap Trading** | Automated buy→wait→sell loop on PumpSwap (outer AMM); configurable rounds and rest | `./run.sh` | [pumpswap_trade](./pumpswap_trade/) |
| **PumpSwap Trading (Encrypted)** | Same as above with encrypted keystore | `./run.sh` | [pumpswap_trade_with_safekey](./pumpswap_trade_with_safekey/) |
| **PumpFun Trading** | Buy→wait→sell on PumpFun bonding curve; token must not have graduated to PumpSwap | `./run.sh` | [pumpfun_trade](./pumpfun_trade/) |
| **PumpFun Trading (Encrypted)** | Same as above; keystore / `KEYPAIR_BASE58` | `./run.sh` | [pumpfun_trade_with_safekey](./pumpfun_trade_with_safekey/) |

### Which example should I use?

| Scenario | Directory |
|----------|-----------|
| Token still on **PumpFun** bonding curve (not graduated to PumpSwap) | `pumpfun_trade` or `pumpfun_trade_with_safekey` |
| Token on **PumpSwap** outer AMM | `pumpswap_trade` or `pumpswap_trade_with_safekey` |
| Private key via `PRIVATE_KEY` or `private_key` in YAML | `pumpfun_trade` / `pumpswap_trade` |
| Encrypted **keystore** + password (or fallback `KEYPAIR_BASE58`) | `pumpfun_trade_with_safekey` / `pumpswap_trade_with_safekey` |

### Shared behavior (all four trading examples)

- ✅ **Flow**: Buy → wait ~30s → sell; **1 round by default** (change `ROUNDS` / `REST_SECS` in each crate’s `src/run.rs`)
- ✅ **Multi-SWQoS**: Concurrent submission to several MEV channels
- ✅ **YAML + `.env`**: `config/dev|prod/solana.yaml`, `trading.yaml`, and env overrides
- ✅ **Durable nonce**: Required when **2+** SWQoS providers are enabled—set `nonce_config` or `NONCE_ACCOUNT`; empty `""` placeholders in YAML are skipped so `NONCE_ACCOUNT` still works
- ✅ **Gas / slippage**: Configurable in `trading.yaml`
- ✅ **Default**: `wait_tx_confirmed: false`; the bot waits a fixed interval after buy before reading balance—tune for production if needed
- ⚠️ **Sell size**: Each round sells the wallet’s **full** token balance for that mint (including any balance you held before the buy)

### Example README index

| Example | Chinese | English |
|---------|---------|---------|
| PumpSwap (private key) | [README_CN.md](./pumpswap_trade/README_CN.md) | [README.md](./pumpswap_trade/README.md) |
| PumpSwap (encrypted) | [README_CN.md](./pumpswap_trade_with_safekey/README_CN.md) | [README.md](./pumpswap_trade_with_safekey/README.md) |
| PumpFun (private key) | [README_CN.md](./pumpfun_trade/README_CN.md) | [README.md](./pumpfun_trade/README.md) |
| PumpFun (encrypted) | [README_CN.md](./pumpfun_trade_with_safekey/README_CN.md) | [README.md](./pumpfun_trade_with_safekey/README.md) |

### Configuration

Each example supports:

- **Environment Variables**: `.env` file for sensitive configuration
- **YAML Config**: `config/dev/solana.yaml` and `config/prod/solana.yaml`
- **Example Templates**: `.yaml.example` files as configuration templates

#### Supported SWQoS Services

| Service | Transport Protocols |
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

**API Key Application**: Apply for API keys through the official website: [https://fnzero.dev/swqos](https://fnzero.dev/swqos)

**Note**: ⚡ = QUIC (Quick UDP Internet Connections) provides lower latency compared to HTTP/gRPC. Services using QUIC (Astralane, Node1, Soyas, Speedlanding) typically offer the best transaction submission performance.

---

## 📦 SDKs & Tools

### sol-trade-sdk

Comprehensive Rust SDK for Solana DEX trading with unified interface for multiple protocols.

**Features:**
- Support for PumpFun, PumpSwap, Bonk, Raydium CPMM, Raydium AMM V4, Meteora DAMM V2
- Multiple MEV protection services (Jito, BlockRazor, Astralane, etc.)
- Middleware system for custom instruction modification
- Shared infrastructure for multi-wallet scenarios
- Address Lookup Table (ALT) support
- Durable Nonce management
- Gas fee strategy optimization

**Documentation:** [sol-trade-sdk/README.md](./sol-trade-sdk/README.md)

### sol-parser-sdk

Transaction parsing SDK with gRPC streaming support for real-time event processing.

**Features:**
- Parse PumpFun, PumpSwap, Raydium, and other DEX transactions
- gRPC-based streaming for low-latency event processing
- Event filtering and transformation
- Trade event extraction
- Account filler for optimized account lookups

**Documentation:** [sol-parser-sdk/README.md](./sol-parser-sdk/README.md)

### sol-safekey

Encrypted key management library for secure Solana keypair storage.

**Features:**
- Encrypt/decrypt Solana keypairs with password protection
- JSON-based keystore format
- Base58 private key support
- CLI tools for key management
- Integration with trading examples

**Documentation:** [sol-safekey/README.md](./sol-safekey/README.md)

### solana-streamer

Utilities for Solana transaction streaming and real-time data processing.

**Features:**
- Shred stream subscription
- Transaction streaming
- Event processing pipeline
- Performance-optimized parsing

**Documentation:** [solana-streamer/README.md](./solana-streamer/README.md)

---

## 🚀 Quick Start

### Prerequisites

- Rust 1.70+ and Cargo
- Solana CLI (optional, for wallet management)
- A Solana RPC endpoint (mainnet-beta or devnet)

### Clone Repository

```bash
git clone https://github.com/0xfnzero/fnzero-examples.git
cd fnzero-examples
```

### Run trading examples

Run commands **inside the example crate directory** (each folder is its own Cargo package; there is no repo-root workspace).

**Option 1: Private key (PumpSwap or PumpFun)**

```bash
cd pumpswap_trade          # outer AMM; use cd pumpfun_trade for bonding-curve tokens

cp .env.example .env
# Edit .env: PRIVATE_KEY, SOLANA_RPC_URL, etc.

cp config/dev/solana.yaml.example config/dev/solana.yaml
# Edit solana.yaml: enable SWQoS, set tokens; for 2+ SWQoS configure nonce or NONCE_ACCOUNT

./run.sh <TOKEN_MINT_ADDRESS>
# or: cargo run --release -- <TOKEN_MINT_ADDRESS>
```

**Option 2: Encrypted keystore**

```bash
cd sol-safekey
cargo run --release -- export <private_key_or_mnemonic> ../pumpswap_trade_with_safekey/keystore.json
cd ../pumpswap_trade_with_safekey   # or pumpfun_trade_with_safekey

cp .env.example .env
cp config/dev/solana.yaml.example config/dev/solana.yaml
# Set keystore_path in solana.yaml (e.g. ./keystore.json)

./run.sh <TOKEN_MINT_ADDRESS>
```

### Configuration Details

#### Environment Variables (.env)

```bash
# Environment: dev or prod
APP_ENV=dev

# Token mint address to trade
MINT=your_token_mint_address

# Sol amount to buy (SOL)
BUY_SOL_AMOUNT=0.01

# RPC URL
SOLANA_RPC_URL=https://your-rpc-endpoint.com

# Keystore password (for encrypted key)
KEYSTORE_PASSWORD=your_password

# Durable nonce account (required for multiple SWQoS)
NONCE_ACCOUNT=your_nonce_account_address
```

#### SWQoS Configuration (config/dev/solana.yaml)

```yaml
swqos:
  region: "Frankfurt"  # or NewYork, Tokyo, etc.
  enabled_providers:
    - provider: "Astralane"
      api_token: "your_token"
      enabled: true
    - provider: "BlockRazor"
      api_token: "your_token"
      enabled: true
    # Add more providers as needed
```

### Build for production

Each example uses `.cargo/config.toml` so artifacts go to **`build-cache/release/`** (not the default `target/release/`).

```bash
cd pumpswap_trade    # or pumpfun_trade, pumpswap_trade_with_safekey, pumpfun_trade_with_safekey

cargo build --release
./build-cache/release/pumpswap_trade_with_safekey <TOKEN_MINT_ADDRESS>

# macOS → Linux bundle (requires x86_64-unknown-linux-gnu toolchain)
./build-linux-release.sh   # produces linux-release/deploy.tar.gz
```

---

## 📄 License

MIT License

See [LICENSE](./LICENSE) for details.

---

## 💬 Contact

- **Official Website**: https://fnzero.dev/
- **Project Repository**: https://github.com/0xfnzero/fnzero-examples
- **Telegram Group**: https://t.me/fnzero_group
- **Discord Server**: https://discord.gg/vuazbGkqQE

---

## ⚠️ Important Notes

1. **Security**: Never commit private keys or API tokens to version control
2. **Testing**: Thoroughly test on devnet before using on mainnet
3. **Risk**: Trading cryptocurrencies involves significant risk
4. **Compliance**: Ensure compliance with local laws and regulations
5. **RPC Limits**: Monitor RPC usage to avoid rate limiting
6. **MEV Services**: Configure API tokens properly for MEV protection services

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## 📚 Additional Resources

- [Solana Documentation](https://docs.solana.com/)
- [Solana Cookbook](https://solanacookbook.com/)
- [Rust Documentation](https://www.rust-lang.org/docs.html)

---

<div align="center">
    <strong>Built with ❤️ by the FnZero team</strong>
</div>
