# PumpFun ShredStream 狙击示例

这个示例使用 `sol-parser-sdk` 的 ShredStream 客户端监听 PumpFun 外层指令，默认只监听创建者首次买入 `is_created_buy=true`，触发后只买入 1 笔，等待 3 秒后仅卖出本次运行新增的代币余额。

不使用 `sol-safekey`。钱包只从 `PRIVATE_KEY` 读取，支持 base58 私钥或 64 字节 JSON 数组。

## 注意

ShredStream 只包含交易外层指令，不包含 gRPC 日志里的完整 TradeEvent 储备字段。示例会从同笔 create 指令缓存 creator 等必要账户，用于买入；卖出前会通过 RPC 刷新 PumpFun 参数，避免使用过期储备。

## 配置

```bash
cp .env.example .env
```

编辑 `.env`：

| 变量 | 说明 |
|------|------|
| `PRIVATE_KEY` | 直接私钥，必填 |
| `RPC_URL` | 交易、余额查询和卖出前参数刷新 RPC |
| `SHREDSTREAM_ENDPOINT` | ShredStream proxy 地址，默认 `http://127.0.0.1:10800` |
| `REQUIRE_CREATED_BUY` | 默认 `true`，只狙击创建者首次买入 |
| `TARGET_MINT` | 可选，只狙击指定 mint |
| `BUY_SOL_AMOUNT` | 每次买入 SOL 数量，默认 `0.01` |
| `BUY_MODE` | `with_max_input`（狙击推荐）或 `exact_input`（固定花费） |
| `BUY_SLIPPAGE_BPS` | 买入滑点，默认 `300` |
| `SELL_SLIPPAGE_BPS` | 卖出滑点，默认 `500`；不要用接近 100% 的值作为常规配置 |
| `HOLD_SECONDS` | 买入后持有秒数，默认 `3` |
| `MAX_EVENT_AGE_MS` | 默认 `1000`，忽略队列中已过期的事件 |
| `WAIT_TX_CONFIRMED` | 自动卖出模式必须为 `true` |
| `WAIT_FOR_ALL_SUBMITS` | 默认 `false`；需要收集全部 SWQoS 提交结果时开启 |
| `ASSUME_PREPARED_ATAS` | 默认 `false`；仅当所需 ATA 已提前准备时设为 `true` |

## 运行

```bash
./run.sh
```

或：

```bash
cargo run --release
```

完成一次买入和一次卖出后程序会退出。

## 低延迟设计

交易客户端和 blockhash cache 都在 ShredStream 订阅前初始化。默认只处理 `is_created_buy=true`，买入协议参数可从 create + 首买上下文构造。事件处理仍会读取一次买前余额，避免后续卖出旧持仓；生产系统可用可信的本地持仓账本替代这次 RPC 查询。若关闭 `REQUIRE_CREATED_BUY` 且 Shred 缺少定价字段，还会增加一次 RPC 回退。

默认 `BUY_MODE=with_max_input`。需要固定花费时可使用 `exact_input`，但必须接受最小输出保护带来的滑点失败。详见 [低延迟 Bot 指南](../LOW_LATENCY_BOT_GUIDE_CN.md)。
