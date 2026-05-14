# PumpFun gRPC 狙击示例

这个示例使用 `sol-parser-sdk` 的 Yellowstone gRPC 订阅 PumpFun 买入事件，默认只监听创建者首次买入 `is_created_buy=true`，触发后只买入 1 笔，等待 3 秒后自动卖出钱包中该 mint 的全部余额。

不使用 `sol-safekey`。钱包只从 `PRIVATE_KEY` 读取，支持 base58 私钥或 64 字节 JSON 数组。

## 配置

```bash
cp .env.example .env
```

编辑 `.env`：

| 变量 | 说明 |
|------|------|
| `PRIVATE_KEY` | 直接私钥，必填 |
| `RPC_URL` | 交易和余额查询 RPC |
| `GRPC_ENDPOINT` | Yellowstone gRPC 地址 |
| `GRPC_AUTH_TOKEN` | gRPC token，无鉴权可留空 |
| `REQUIRE_CREATED_BUY` | 默认 `true`，只狙击创建者首次买入 |
| `TARGET_MINT` | 可选，只狙击指定 mint |
| `BUY_SOL_AMOUNT` | 每次买入 SOL 数量，默认 `0.01` |
| `BUY_SLIPPAGE_BPS` | 买入滑点，默认 `300` |
| `SELL_SLIPPAGE_BPS` | 卖出滑点，默认 `9980` |
| `HOLD_SECONDS` | 买入后持有秒数，默认 `3` |
| `WAIT_TX_CONFIRMED` | 默认 `true`，买卖等待确认后返回 |

## 运行

```bash
./run.sh
```

或：

```bash
cargo run --release
```

完成一次买入和一次卖出后程序会退出。
