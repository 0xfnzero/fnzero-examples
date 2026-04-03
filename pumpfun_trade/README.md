<div align="center">
    <h1>🎯 PumpFun Bonding-Curve Trading Example</h1>
    <h3><em>Buy → wait → sell on PumpFun with multiple SWQoS channels</em></h3>
</div>

<p align="center">
    <strong>For tokens still on the PumpFun bonding curve (not graduated to PumpSwap). Configure the wallet via <code>PRIVATE_KEY</code> or <code>private_key</code> in <code>config/*/solana.yaml</code>.</strong>
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
    <a href="../README.md">Repository overview</a>
</p>

> **Note**: After the token **graduates to PumpSwap**, use **`pumpswap_trade`** / **`pumpswap_trade_with_safekey`**.

---

<p align="center"><a href="../README.md">← Back to repository overview</a></p>

---

## 📋 Table of contents

- When to use · Features · Installation · Configuration · Run · Build
- Comparison with PumpSwap example

---

## When to use this example

| Use `pumpfun_trade` | Use `pumpswap_trade` instead |
|---------------------|-------------------------------|
| Token is still on the **PumpFun** bonding curve | Token has **graduated** to PumpSwap |
| `PumpFunParams::from_mint_by_rpc` succeeds | You need a **pool address** for PumpSwap |

If you pass a graduated mint here, the program fails while resolving the bonding curve.

---

## Features

1. **Protocol**: `DexType::PumpFun` with `PumpFunParams::from_mint_by_rpc`; params are **fetched again before sell** so `creator_vault` and curve state stay fresh.
2. **Flow**: Buy → wait ~30s → sell the wallet’s **full balance** of that mint; **1 round** by default (edit `ROUNDS`, `REST_SECS` in `src/run.rs`).
3. **Multi-SWQoS**, **durable nonce** (required when 2+ SWQoS are enabled), **trading.yaml** for slippage/Gas—same ideas as `pumpswap_trade`.
4. **Nonce placeholders**: empty strings in `nonce_config` lists are ignored so `NONCE_ACCOUNT` can still apply.

---

## Installation

```bash
git clone https://github.com/0xfnzero/fnzero-examples.git
cd fnzero-examples/pumpfun_trade
```

Install Rust/Cargo: [https://rustup.rs](https://rustup.rs)

---

## Configuration

**Before first run**, create local files from templates (do not commit; see repo root [README.md](../README.md) “Before you run & privacy”):

```bash
cp .env.example .env
cp config/dev/solana.yaml.example config/dev/solana.yaml
cp config/dev/trading.yaml.example config/dev/trading.yaml
# For prod, copy config/prod/*.example as well
```

Then edit `.env` and YAML: `PRIVATE_KEY`, RPC, SWQoS tokens, `nonce_config`, etc. Trading params are in `trading.yaml`.

For more wallet/SWQoS detail, see [pumpswap_trade/README.md](../pumpswap_trade/README.md) (protocol/mint requirements differ).

---

## Run

```bash
./run.sh <MINT_ADDRESS>
# or
cargo run --release -- <MINT_ADDRESS>
```

`APP_ENV=dev|prod` selects `config/dev` vs `config/prod`.

---

## Build

`.cargo/config.toml` writes artifacts to **`build-cache/release/`**.

```bash
cargo build --release
./build-cache/release/pumpfun_trade <MINT_ADDRESS>
```

Linux bundle:

```bash
./build-linux-release.sh
```

---

## Comparison with PumpSwap example

| | `pumpfun_trade` | `pumpswap_trade` |
|---|-----------------|------------------|
| DEX | PumpFun (curve) | PumpSwap (outer AMM) |
| Pool | Derived from mint / curve | `find_pool_by_mint` |
| Mint | Not graduated | Listed on outer pool |

Encrypted-keystore variant: [pumpfun_trade_with_safekey/README.md](../pumpfun_trade_with_safekey/README.md).

---

## 📄 License

MIT License

---

## 💬 Contact

- Website: https://fnzero.dev/
- Repository: https://github.com/0xfnzero/fnzero-examples
