<div align="center">
    <h1>🔄 PumpSwap Auto-Trading Example</h1>
    <h3><em>Automated buy-wait-sell loop trading on PumpSwap with multiple SWQoS</em></h3>
</div>

<p align="center">
    <strong>A Rust example demonstrating automated trading strategy on PumpSwap: buy → wait 30s → sell → wait 30s, repeat for 3 rounds. Supports multiple SWQoS services for concurrent transaction submission.</strong>
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
    <a href="https://fnzero.dev/">Website</a> |
    <a href="https://t.me/fnzero_group">Telegram</a> |
    <a href="https://discord.gg/vuazbGkqQE">Discord</a>
</p>

---

## 📋 Table of Contents

- [✨ Features](#-features)
- [📦 Installation](#-installation)
- [🛠️ Configuration](#️-configuration)
  - [Environment Variables](#environment-variables)
  - [Config Files](#config-files)
  - [SWQoS Services](#swqos-services)
- [🚀 Usage](#-usage)
- [🔧 Build](#-build)
- [📄 License](#-license)
- [💬 Contact](#-contact)

---

## ✨ Features

1. **Automated Trading Loop**: Buy → Wait 30s → Sell → Wait 30s, repeat for 3 rounds
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
cd fnzero-examples/pumpswap_buy_sell_swqos
```

### Dependencies

Ensure you have Rust and Cargo installed:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

---

## 🛠️ Configuration

### Environment Variables

Create a `.env` file based on `.env.example`:

```bash
# Environment: dev or prod
APP_ENV=prod

# Token mint address (can also be passed as CLI argument)
MINT=

# Keystore password (optional, interactive if not set)
KEYSTORE_PASSWORD=

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

#### Creating keystore.json

**Option 1: Using sol-safekey (Recommended)**

```bash
# Install sol-safekey
cargo install sol-safekey

# Generate encrypted keystore
sol-safekey new keystore.json

# Enter password when prompted (10-20 characters recommended)
```

**Option 2: Converting existing keypair**

```bash
# From base58 private key
echo "your_base58_private_key" > key.txt
sol-safekey import key.txt keystore.json

# From JSON array (64 bytes)
cat keypair.json | sol-safekey import - keystore.json
```

#### Creating Required Accounts

**1. WSOL ATA (Wrapped SOL Associated Token Account)**

WSOL ATA is automatically created during the first buy operation. No manual setup required.

**2. Durable Nonce Account**

Durable nonce accounts are required when using 2 or more SWQoS services for transaction replay protection.

```bash
# Install Solana CLI
sh -c "$(curl -sSfL https://release.solana.com/stable/install)"

# Generate durable nonce keypair
solana-keygen new --outfile nonce-keypair.json

# Create nonce account
solana create-nonce-account nonce-keypair.json

# Get nonce address
solana-keygen pubkey nonce-keypair.json
```

Add the nonce address to your configuration:

```yaml
# config/dev/solana.yaml or config/prod/solana.yaml
nonce_config:
  buy_nonce_accounts:
    - "YOUR_NONCE_PUBKEY_HERE"
  sell_nonce_accounts:
    - "YOUR_NONCE_PUBKEY_HERE"
```

**Note**: You can use separate nonce accounts for buy and sell, or use the same one.

### Config Files

Configuration is organized in `config/` directory:

```
config/
├── dev/
│   ├── solana.yaml      # RPC, keystore, SWQoS, nonce config
│   └── trading.yaml    # Trading parameters, gas fee settings
└── prod/
    ├── solana.yaml
    └── trading.yaml
```

**Environment Variables Override**:
- All sensitive values in config files can be overridden by environment variables
- `.env` values have higher priority than config files
- Useful for keeping secrets out of version control

### SWQoS Services

The following SWQoS services are supported:

| Service | Required Param | Website |
|---------|----------------|----------|
| Astralane | API Key | https://astralane.io |
| BlockRazor | API Key | https://blockrazor.io |
| Jito | UUID | https://jito.wtf |
| NextBlock | API Key | - |
| Bloxroute | API Key | https://www.bloxroute.com |
| ZeroSlot | API Key | https://zeroslot.com |
| Temporal | API Key | https://temporal.cloud |
| FlashBlock | API Key | - |
| Node1 | API Key | - |

Apply for API keys at: https://fnzero.dev/swqos

---

## 🚀 Usage

### Run with CLI Argument

```bash
./pumpswap_buy_sell_swqos <MINT_ADDRESS>
```

### Run with Environment Variable

```bash
MINT=<MINT_ADDRESS> ./pumpswap_buy_sell_swqos
```

### Using .env File

```bash
# Set MINT in .env or pass as argument
./pumpswap_buy_sell_swqos
```

### Using Different Environment

```bash
APP_ENV=prod ./pumpswap_buy_sell_swqos <MINT_ADDRESS>
```

---

## 🔧 Build

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
├── x86_64-unknown-linux-gnu/release/pumpswap_buy_sell_swqos
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
