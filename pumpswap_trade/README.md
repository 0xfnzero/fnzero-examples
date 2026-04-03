<div align="center">
    <h1>🔄 PumpSwap Auto-Trading Example</h1>
    <h3><em>Automated buy-wait-sell loop trading on PumpSwap with multiple SWQoS</em></h3>
</div>

<p align="center">
    <strong>Rust example: automated buy → wait ~30s → sell on PumpSwap outer AMM (1 round by default; edit <code>src/run.rs</code>). Multiple SWQoS channels.</strong>
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
    <a href="https://fnzero.dev/">Website</a> |
    <a href="https://t.me/fnzero_group">Telegram</a> |
    <a href="https://discord.gg/vuazbGkqQE">Discord</a>
</p>

> **Note**: If the token is still on **PumpFun** (bonding curve, not graduated), use **`pumpfun_trade`** / **`pumpfun_trade_with_safekey`** in this repo instead.

---

<p align="center"><a href="../README.md">← Back to repository overview</a></p>

---

## 📋 Table of Contents

- [✨ Features](#-features)
- [📦 Installation](#-installation)
- [🛠️ Configuration](#️-configuration)
  - [Environment Variables](#environment-variables)
  - [Wallet Setup](#wallet-setup)
  - [Config Files](#config-files)
  - [SWQoS Services](#swqos-services)
- [🚀 Usage](#-usage)
- [🔧 Build](#-build)
- [📄 License](#-license)
- [💬 Contact](#-contact)

---

## ✨ Features

1. **Automated Trading Loop**: Buy → wait ~30s → sell; **1 round** by default (edit `ROUNDS`, `REST_SECS` in `src/run.rs`)
2. **Multiple SWQoS Support**: Send transactions concurrently to multiple MEV protection services
3. **Durable Nonce**: Transaction replay protection for multi-SWQoS scenarios
4. **Flexible Configuration**: Support for both YAML config files and environment variables
5. **Environment Priority**: Environment variables override config file settings for sensitive data
6. **Cross-Platform Build**: Build scripts for Linux deployment

---

## 📦 Installation

### Clone Repository

```bash
git clone https://github.com/0xfnzero/fnzero-examples.git
cd fnzero-examples/pumpswap_trade
```

### Dependencies

Ensure you have Rust and Cargo installed:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

---

## 🛠️ Configuration

### First-time setup (privacy)

The repo does **not** track your `.env`, `config/**/solana.yaml`, or `config/**/trading.yaml` (see root [README.md](../README.md) “Before you run & privacy”). On first use:

```bash
cp .env.example .env
cp config/dev/solana.yaml.example config/dev/solana.yaml
cp config/dev/trading.yaml.example config/dev/trading.yaml
```

Edit those files with keys, RPC, SWQoS tokens—**never** `git add` them.

### Environment Variables

Create a `.env` file based on `.env.example`:

```bash
# Environment: dev or prod
APP_ENV=prod

# Token mint address (can also be passed as CLI argument)
MINT=

# Wallet private key (supports base58 or standard 64-byte array JSON format)
PRIVATE_KEY=

# RPC URL
SOLANA_RPC_URL=http://your-rpc-endpoint.com

# Buy amount in SOL
BUY_SOL_AMOUNT=0.01

# SWQoS region
SWQOS_REGION=Frankfurt

# SWQoS provider tokens
SWQOS_ASTRALANE_TOKEN=
SWQOS_BLOCKRAZOR_TOKEN=
SWQOS_JITO_TOKEN=
SWQOS_ZEROSLOT_TOKEN=
# ... etc.

# Durable nonce account (required when using 2+ SWQoS)
NONCE_ACCOUNT=
```

### Wallet Setup

This project supports direct private key configuration. You can configure your wallet in two ways:

#### Option 1: Using .env File (Recommended)

1. Copy `.env.example` to `.env`
2. Add your private key to the `PRIVATE_KEY` variable

```bash
cp .env.example .env
```

Edit `.env`:
```bash
PRIVATE_KEY=your_private_key_here
```

**Private Key Format Support:**
- **Base58 format**: Your standard Solana private key string (e.g., from `solana-keygen new`)
- **64-byte array JSON**: `[1,2,3,...64]` format exported from some wallets

#### Option 2: Using Config File

You can also set the private key directly in the config file:

```yaml
# config/dev/solana.yaml or config/prod/solana.yaml
private_key: "your_private_key_here"
```

**Security Notes:**
- Never commit `.env` files to version control
- Use environment variables in production for better security
- Keep your private key safe and never share it

### Config Files

Keep local config (copied from `*.example`, **not committed**) under `config/`:

```
config/
├── dev/
│   ├── solana.yaml      # local: RPC, private_key, SWQoS, nonce
│   └── trading.yaml     # local: trading params, gas
└── prod/
    ├── solana.yaml
    └── trading.yaml
```

Only `*.yaml.example` templates live in the repository.

**Environment Variables Override**:
- All sensitive values in config files can be overridden by environment variables
- `.env` values have higher priority than config files
- Useful for keeping secrets out of version control

### SWQoS Services

The following SWQoS services are supported:

| Service | Required Param |
|---------|----------------|
| Astralane | API Key |
| BlockRazor | API Key |
| Jito | UUID |
| NextBlock | API Key |
| Bloxroute | API Key |
| ZeroSlot | API Key |
| Temporal | API Key |
| FlashBlock | API Key |
| Node1 | API Key |

Apply for API keys at: https://fnzero.dev/swqos

---

## 🚀 Usage

### Quick Start

Simply run the script:

```bash
./run.sh
```

The script will:
- Load configuration from `.env` and `config/dev/solana.yaml`
- Build and run the trading bot in release mode
- Support dev/prod environments via `APP_ENV` variable

### Run with CLI Argument

```bash
./pumpswap_trade <MINT_ADDRESS>
```

### Run with Environment Variable

```bash
MINT=<MINT_ADDRESS> ./pumpswap_trade
```

### Using .env File

```bash
# Set MINT in .env or pass as argument
./pumpswap_trade
```

### Using Different Environment

```bash
APP_ENV=prod ./pumpswap_trade <MINT_ADDRESS>
```

---

## 🔧 Build

### Build for Local Development

```bash
cargo build --release
```

This crate sets `target-dir = "build-cache"` in `.cargo/config.toml`; the binary is at **`build-cache/release/pumpswap_trade`**

### Build for Linux Release

```bash
./build-linux-release.sh
```

This script:
1. Cross-compiles for Linux (x86_64-unknown-linux-gnu)
2. Packages binary and configs into `linux-release/deploy.tar.gz`

Output:
```
linux-release/
├── x86_64-unknown-linux-gnu/release/pumpswap_trade
└── deploy.tar.gz
```

---

## 📄 License

MIT License

---

## 💬 Contact

- Official Website: https://fnzero.dev/
- Project Repository: https://github.com/0xfnzero/fnzero-examples
- Telegram Group: https://t.me/fnzero_group
- Discord: https://discord.gg/vuazbGkqQE
