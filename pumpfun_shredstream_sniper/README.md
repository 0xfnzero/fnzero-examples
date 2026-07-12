# PumpFun ShredStream Sniper Example

This example uses the `sol-parser-sdk` ShredStream client to monitor PumpFun outer instructions. By default it waits for the creator first buy (`is_created_buy=true`), buys exactly once, waits 3 seconds, then sells only the token balance added by this run.

It does not use `sol-safekey`. The wallet is loaded directly from `PRIVATE_KEY` as either a base58 key or a 64-byte JSON array.

## Note

ShredStream does not include the full gRPC log TradeEvent reserve fields. This example caches required accounts from the same transaction's create instruction for the buy path, then refreshes PumpFun parameters through RPC before selling.

## Setup

```bash
cp .env.example .env
```

Edit `.env`:

| Variable | Description |
|---|---|
| `PRIVATE_KEY` | Required base58 key or 64-byte JSON array |
| `RPC_URL` | RPC used for trading, balances, and sell-state refresh |
| `SHREDSTREAM_ENDPOINT` | ShredStream proxy; default `http://127.0.0.1:10800` |
| `REQUIRE_CREATED_BUY` | Default `true`; match creator first buys only |
| `TARGET_MINT` | Optional mint allowlist entry |
| `BUY_SOL_AMOUNT` | SOL buy size; default `0.01` |
| `BUY_MODE` | `with_max_input` (recommended for sniping) or `exact_input` |
| `BUY_SLIPPAGE_BPS` | Buy slippage; default `300` |
| `SELL_SLIPPAGE_BPS` | Sell slippage; default `500`; near-100% values are not routine settings |
| `HOLD_SECONDS` | Hold time after buy; default `3` |
| `MAX_EVENT_AGE_MS` | Default `1000`; stale queued events are ignored |
| `WAIT_TX_CONFIRMED` | Must be `true` for automatic selling |
| `WAIT_FOR_ALL_SUBMITS` | Default `false`; enable only to collect all route results |
| `ASSUME_PREPARED_ATAS` | Default `false`; enable only when every required ATA exists |

## Run

```bash
./run.sh
```

Or run `cargo run --release`.

The process exits after one buy and one sell.

## Low-latency design

The trading client and blockhash cache are initialized before ShredStream subscription. The default `REQUIRE_CREATED_BUY=true` path builds protocol state from create plus creator-first-buy context. It still reads the pre-buy token balance so the later sell cannot liquidate older holdings; production systems can replace this read with a trusted local position ledger. Disabling `REQUIRE_CREATED_BUY` can add another RPC fallback when shred data lacks pricing fields.

A production bot still needs durable signature deduplication, a position state machine, and bounded requoting. See the [low-latency bot guide](../LOW_LATENCY_BOT_GUIDE.md).
