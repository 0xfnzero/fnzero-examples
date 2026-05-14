# PumpFun ShredStream Sniper Example

This example uses the `sol-parser-sdk` ShredStream client to monitor PumpFun outer instructions. By default it waits for the creator first buy (`is_created_buy=true`), buys exactly once, waits 3 seconds, then sells the wallet's full balance for that mint.

It does not use `sol-safekey`. The wallet is loaded directly from `PRIVATE_KEY` as either a base58 key or a 64-byte JSON array.

## Note

ShredStream does not include the full gRPC log TradeEvent reserve fields. This example caches required accounts from the same transaction's create instruction for the buy path, then refreshes PumpFun parameters through RPC before selling.

## Setup

```bash
cp .env.example .env
```

Edit `.env`, especially `PRIVATE_KEY`, `RPC_URL`, and `SHREDSTREAM_ENDPOINT`.

## Run

```bash
./run.sh
```

The process exits after one buy and one sell.
