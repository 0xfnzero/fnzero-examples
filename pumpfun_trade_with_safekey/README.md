<div align="center">
    <h1>🎯 PumpFun Trading Example (Encrypted Keystore)</h1>
    <h3><em>Keystore / KEYPAIR_BASE58 + multi-SWQoS</em></h3>
</div>

<p align="center">
    <strong>Same trading flow as <a href="../pumpfun_trade/README.md">pumpfun_trade</a>, but the wallet uses a <code>sol-safekey</code> file or fallback <code>KEYPAIR_BASE58</code>.</strong>
</p>

<p align="center">
    <a href="README_CN.md">中文</a> |
    <a href="README.md">English</a> |
    <a href="../README.md">Repository overview</a>
</p>

---

<p align="center"><a href="../README.md">← Back to repository overview</a></p>

---

## When to use

- Token must still be on **PumpFun** (not graduated); use `pumpswap_trade_with_safekey` after migration.
- Set **`keystore_path`** in `config/*/solana.yaml` (e.g. `./keystore.json`), enter the password at runtime or set **`KEYSTORE_PASSWORD`**.
- If `keystore_path` is empty, **`KEYPAIR_BASE58`** is accepted as a fallback (less secure).

## Create a keystore

```bash
cd sol-safekey
cargo run --release -- export <private_key_or_mnemonic> ../pumpfun_trade_with_safekey/keystore.json
```

Then point `keystore_path` in `pumpfun_trade_with_safekey/config/dev/solana.yaml` to that file.

For longer documentation (SWQoS, nonce, security), see [pumpswap_trade_with_safekey/README.md](../pumpswap_trade_with_safekey/README.md)—same layout, different protocol (PumpSwap).

## Before you run

1. Clone **[sol-safekey](https://github.com/0xfnzero/sol-safekey)** separately and export `keystore.json` (this repo does not vendor it)—see root [README.md](../README.md).
2. In this crate:

```bash
cp .env.example .env
cp config/dev/solana.yaml.example config/dev/solana.yaml
cp config/dev/trading.yaml.example config/dev/trading.yaml
```

3. Edit YAML/`keystore_path`, SWQoS, nonce. **Do not commit** `.env` or filled YAML.

## Run & build

```bash
cd pumpfun_trade_with_safekey
./run.sh <MINT_ADDRESS>
```

```bash
cargo build --release
# Binary: build-cache/release/pumpfun_trade_with_safekey
./build-linux-release.sh
```

## Behavior summary

- **1 round** by default: buy → wait ~30s → sell full mint balance (`ROUNDS` in `src/run.rs`).
- **2+ SWQoS** requires durable nonce; empty YAML placeholders are skipped so **`NONCE_ACCOUNT`** still works.
- Params are refreshed from RPC before sell, same as `pumpfun_trade`.

---

## 📄 License

MIT License

## 💬 Contact

https://fnzero.dev/ · https://github.com/0xfnzero/fnzero-examples
