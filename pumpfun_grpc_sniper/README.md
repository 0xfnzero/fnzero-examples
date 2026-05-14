# PumpFun gRPC Sniper Example

This example uses `sol-parser-sdk` Yellowstone gRPC to monitor PumpFun buy events. By default it waits for the creator first buy (`is_created_buy=true`), buys exactly once, waits 3 seconds, then sells the wallet's full balance for that mint.

It does not use `sol-safekey`. The wallet is loaded directly from `PRIVATE_KEY` as either a base58 key or a 64-byte JSON array.

## Setup

```bash
cp .env.example .env
```

Edit `.env`, especially `PRIVATE_KEY`, `RPC_URL`, and `GRPC_ENDPOINT`.

## Run

```bash
./run.sh
```

The process exits after one buy and one sell.
