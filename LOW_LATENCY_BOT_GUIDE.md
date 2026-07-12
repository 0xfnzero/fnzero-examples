# FnZero Low-Latency Bot Guide

This guide defines the boundary between event ingestion, parsing, and `sol-trade-sdk`. The examples demonstrate correct integration patterns; they do not replace application-specific position management, risk controls, and monitoring.

## Component selection

| Need | Component |
|---|---|
| Yellowstone gRPC or ShredStream with unified DEX events | `sol-parser-sdk` |
| An existing `solana-streamer-sdk` event pipeline | Keep streamer as input and map event fields into trade params |
| Transaction construction, signing, RPC, and SWQoS submission | `sol-trade-sdk` |

`sol-parser-sdk` and `solana-streamer-sdk` are alternative event entry points. Consuming both requires global deduplication by signature and instruction index.

## Process boundary

Before subscription, initialize the wallet, `SolanaTrade`, SWQoS clients, blockhash cache or nonce pool, known ATAs, and ALTs. Restore deduplication and position state before accepting events.

The initial trade hot path should be limited to:

```text
filter -> deduplicate -> reject stale event -> map post-trade state -> build -> sign -> submit
```

Do not initialize `SolanaTrade`, fetch a blockhash, query a balance, or search for a pool in this path. An RPC fallback is valid when shred data is incomplete, but that path is no longer purely low latency.

## State and fees

- Use post-trade reserves from the event. Never reuse a pre-buy snapshot for a later sell.
- PumpFun params must preserve quote mint, creator, creator vault, token program, cashback, and mayhem fields.
- PumpSwap should use `PumpSwapParams::from_trade_with_fee_basis_points` with current reserves and LP, protocol, creator, and cashback fee data.
- Refresh delayed sells from a continuously updated pool cache or RPC.

## Trade intent

| Goal | `BuyAmount` | Protection |
|---|---|---|
| Spend an exact quote amount | `ExactInput(amount)` | Minimum output |
| Prioritize fills for sniping/arbitrage | `WithMaxInput { quote_amount }` | Maximum quote cost |
| Receive an exact token amount | `ExactOutput { output_amount, max_input_amount }` | Maximum input |

`WithMaxInput` does not disable slippage protection. It moves protection from minimum output to maximum input. Never use `min_out = 0` as routine error handling.

Sells normally use `SellAmount::ExactInput(balance)`. Slippage near `10_000 bps` is effectively unprotected and must be treated as an explicit emergency-exit policy.

## Accounts and submission

- Use `AccountPolicy::Auto` when a new mint ATA cannot exist before the event.
- Use `HotPathMinimal` or `AssumePrepared` only for accounts prepared before subscription.
- Use an ALT or pre-created accounts when V2 plus nonce, tip, and ATA instructions approach packet limits.
- Refresh recent blockhashes in the background. A durable nonce extends transaction validity but does not preserve quote freshness.
- Keep `wait_for_all_submits=false` for fastest-return behavior; enable it only when every route result is needed.

## Bounded requoting

For `BuySlippageBelowMinBaseAmountOut` or an equivalent error, discard the old transaction, obtain newer reserves and fee rates, enforce an event-age limit, and rebuild at most the configured number of times. Do not resend the same quote with progressively larger slippage when state did not advance.

## Examples

| Example | Event input | Trade path |
|---|---|---|
| `pumpfun_grpc_sniper` | `sol-parser-sdk` Yellowstone gRPC | Prewarmed client, blockhash cache, `Simple*Params` |
| `pumpfun_shredstream_sniper` | `sol-parser-sdk` ShredStream | Create/first-buy context without initial-buy RPC |
| `sol-trade-sdk/examples/pumpswap_trading` | `solana-streamer-sdk` gRPC | Dynamic event fees and refreshed sell state |
| `pumpfun_trade` / `pumpswap_trade` | RPC-driven | Full buy/sell, SWQoS, and nonce configuration |

The `*_with_safekey` crates use `sol-safekey 0.1.8` from crates.io and the same `sol-trade-sdk 4.0.22` high-level trade API as the direct-key loop examples. Keystore decryption belongs in startup, never in the event hot path.

Production deployments must also add position isolation, maximum loss/input limits, target allowlists, persistent signature tracking, reconnect deduplication, graceful shutdown, and alerts.
