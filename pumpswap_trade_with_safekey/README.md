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
    <a href="README_CN.md">中文</a> |
    <a href="README.md">English</a> |
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
cd fnzero-examples/pumpswap_trade_with_safekey
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

#### Complete Wallet Setup Guide (Recommended)

This guide will walk you through setting up everything using **sol-safekey**, including creating keystore.json, unlocking the wallet, creating WSOL ATA, and creating durable nonce accounts.

##### Step 1: Install sol-safekey

**Install from source (Recommended):**

```bash
# 1. Clone or navigate to sol-safekey project directory
cd /path/to/sol-safekey

# 2. Build and install with full feature
cargo install --path . --features="full"

# 3. Verify installation
sol-safekey --version
```

**Install from crates.io:**

```bash
cargo install sol-safekey --features="full"
```

##### Step 2: Start Interactive Menu

```bash
sol-safekey start
```

You will see the language selection screen:

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  Sol-SafeKey - Solana Security Key Management Tool
  Solana 密钥管理工具
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Please select language | 请选择语言:

  1. 中文
  2. English

Select language (1-2): _
```

Enter `2` to select English.



##### Step 3: Create keystore.json Wallet

After selecting language, you'll see the main menu:

```

==================================================
  Sol-SafeKey - Solana Key Management Tool
==================================================

Core Functions (3 operations):

  1.  Create Plaintext Key
  2.  Create Encrypted Key (bot)
  3.  Decrypt Key

  🔒 Wallet Status: Unlocked
  U.  Unlock Wallet (for Solana Operations)

  Advanced Security Features:
  4.  Setup 2FA Authentication
  5.  Generate Triple-Factor Wallet
  6.  Unlock Triple-Factor Wallet

  Solana On-Chain Operations:
  7.  Check SOL Balance
  8.  Transfer SOL
  9.  Create WSOL ATA
  10.  Wrap SOL → WSOL
  11.  Unwrap WSOL → SOL
  12.  Close WSOL ATA
  13.  Transfer SPL Token
  14.  Create Nonce Account
  15.  Pump.fun Sell Token
  16.  PumpSwap Sell Token
  17.  Pump.fun Cashback (View & Claim)
  18.  PumpSwap Cashback (View & Claim)

  0.  Exit

Select operation (0-18/U): _
```
**Important Note**: Since you haven't created a wallet yet, you need to **unlock wallet** or **create a new wallet** first.

**Steps to Create New Wallet:**
1. Select `1. Create Plaintext Key`
2. Follow prompts to generate keypair
3. Follow prompts to save to file

**Steps to Encrypt Existing Key:**
1. Select `2. Create Encrypted Key`
2. Enter or paste existing private key
3. Enter password when prompted
4. Enter password to confirm
5. Select filename to save (default: keystore.json)

**Steps to Unlock Wallet:**
1. Select `U` Unlock Wallet
2. Enter keystore file path when prompted (default: keystore.json)
3. Enter password
4. After unlock, wallet is stored in session for Solana operations

**Example Output:**
```
  Unlock Wallet
Keystore file path [keystore.json]:

Enter password: ********

✅ Wallet unlocked successfully!
📍 Current Wallet: 7xKm...9xW3
```

##### Step 4: Create WSOL ATA

After unlocking wallet, you need to create a WSOL Associated Token Account. However, WSOL ATA is automatically created during the first buy operation, so you don't need to manually create it.

If you want to create it manually for testing:

1. From the main menu, enter the Solana operations
2. Select "Create WSOL ATA"
3. System will automatically create WSOL ATA

**Note**: For the pumpswap_trade_with_safekey example, WSOL ATA is automatically created during the first buy operation, so manual setup is not required.

##### Step 5: Create Durable Nonce Account

Create a nonce account for transaction replay protection (required when using 2+ SWQoS services):

1. From Solana operations menu, select "Create Nonce Account"
2. System will create a new nonce account
3. Wait for transaction confirmation
4. **Important**: Save the created nonce account address for configuration

**Example Output:**
```
🚀 Creating Nonce Account...

✅ Nonce account created and initialized successfully!
   📍 Address: 5xKm...7xW3
   🔐 Nonce value: 1234abcd...efgh5678

💡 Please save this Nonce account address for future use!
```

##### Step 6: Configure Nonce Account

Add the nonce account address created above to your configuration file:

```yaml
# config/dev/solana.yaml or config/prod/solana.yaml
nonce_config:
  buy_nonce_accounts:
    - "5xKm...7xW3"  # Use the actual created address
  sell_nonce_accounts:
    - "5xKm...7xW3"  # Can use the same, or create different ones
```

**Notes:**
- WSOL ATA is automatically created during the first buy operation, no manual setup required
- When using 2 or more SWQoS services, durable nonce accounts are required
- You can use separate nonce accounts for buy and sell operations, or use the same one

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

### Run with CLI Argument

```bash
./pumpswap_trade_with_safekey <MINT_ADDRESS>
```

### Run with Environment Variable

```bash
MINT=<MINT_ADDRESS> ./pumpswap_trade_with_safekey
```

### Using .env File

```bash
# Set MINT in .env or pass as argument
./pumpswap_trade_with_safekey
```

### Using Different Environment

```bash
APP_ENV=prod ./pumpswap_trade_with_safekey <MINT_ADDRESS>
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
├── x86_64-unknown-linux-gnu/release/pumpswap_trade_with_safekey
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
