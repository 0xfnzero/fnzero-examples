# PumpFun gRPC 狙击示例

这个示例使用 `sol-parser-sdk` 的 Yellowstone gRPC 订阅 PumpFun 买入事件，默认只监听创建者首次买入 `is_created_buy=true`，触发后只买入 1 笔，等待 3 秒后仅卖出本次运行新增的代币余额。

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

示例在订阅前初始化 `SolanaTrade`，并在后台每 400ms 更新 blockhash。事件到达后不再同步初始化客户端或查询 blockhash。默认 `BUY_MODE=with_max_input`，滑点保护最大 SOL 成本；切换到 `exact_input` 后会保护最小代币输出，在活跃市场中更容易出现滑点错误。

事件处理仍会读取一次买前余额，确保后续卖出不会清算本次运行之前已有的持仓；生产系统可用可信的本地持仓账本替代这次 RPC 查询。生产机器人仍应增加持久化事件签名去重、持仓状态机和有限次数重新报价。详见 [低延迟 Bot 指南](../LOW_LATENCY_BOT_GUIDE_CN.md)。
