# FnZero 低延迟 Bot 开发指南

本指南说明如何组合事件流、解析器和 `sol-trade-sdk`。示例用于展示正确边界，不替代业务自己的仓位、风控和监控系统。

## 组件选择

| 需求 | 推荐组件 |
|---|---|
| Yellowstone gRPC 或 ShredStream 订阅并直接得到统一 DEX 事件 | `sol-parser-sdk` |
| 已使用 `solana-streamer-sdk` 的事件管线 | 保留 streamer 作为输入，事件字段映射到 `sol-trade-sdk` params |
| 构建、签名、RPC/SWQoS 提交交易 | `sol-trade-sdk` |

`sol-parser-sdk` 和 `solana-streamer-sdk` 是两套可选事件入口。不要对同一交易同时消费两套入口，除非使用签名 + 指令索引做全局去重。

## 正确的进程边界

启动阶段可以进行 RPC：

1. 加载钱包和风控配置。
2. 创建并预热 `SolanaTrade`、RPC 和 SWQoS 客户端。
3. 启动后台 blockhash cache，或准备 durable nonce pool。
4. 为已知 mint 准备 ATA、WSOL ATA 和 ALT。
5. 建立事件订阅，恢复签名去重和持仓状态。

首次抢单热路径应限制为：

```text
事件过滤 -> 签名去重 -> 检查事件年龄 -> 映射成交后状态 -> 构建 -> 签名 -> 提交
```

不要在事件热路径创建 `SolanaTrade`、查询 blockhash、查余额或搜索池。Shred 数据缺少必需字段时，RPC 回退是正确性优先路径，但不再是纯低延迟路径。

## 事件状态

- 使用事件中的成交后 reserves。不要用指令执行前的余额，也不要在自己的买入后继续复用买入前快照。
- PumpFun 需要正确传入 `quote_mint`、creator、creator vault、token program、cashback 和 mayhem 字段。
- PumpSwap 优先使用 `PumpSwapParams::from_trade_with_fee_basis_points`，传入 base/quote reserves、LP fee、protocol fee、creator fee 和 cashback 信息。
- 卖出发生在几秒之后时，应使用最新流事件更新的池缓存；没有池缓存时，通过 RPC 刷新后再卖。

## 买入模式

| 目标 | `BuyAmount` | 保护方式 |
|---|---|---|
| 固定花费 quote | `ExactInput(amount)` | 最小输出；活跃市场可能报 6040/同类错误 |
| 狙击或套利优先成交 | `WithMaxInput { quote_amount }` | 最大 quote 成本 |
| 固定买到 token 数量 | `ExactOutput { output_amount, max_input_amount }` | 最大输入 |

`WithMaxInput` 不等于无滑点保护。它把保护从最小输出切换为最大输入。不要把 `min_out` 设为零。

卖出通常使用 `SellAmount::ExactInput(balance)`。接近 `10_000 bps` 的卖出滑点接近取消保护，只应作为明确标注的紧急退出策略。

## ATA 和交易体积

- 新币狙击通常无法提前创建目标 mint ATA，使用 `AccountPolicy::Auto`。
- 已知 mint 且 ATA 已准备时使用 `HotPathMinimal` 或 `AssumePrepared`。
- PumpFun V2、durable nonce、tip 和 ATA 创建组合可能超过包大小；提前准备 ATA 或配置 ALT。
- `ASSUME_PREPARED_ATAS=true` 不是性能开关，账户不存在时交易会失败。

## Blockhash、nonce 和多路发送

- 单路或可接受多份 recent-blockhash 交易时，后台每个 slot 附近刷新 blockhash，事件处理只读取缓存。
- 多 SWQoS 发送同一 durable-nonce 交易时，只有一个版本能成功消费 nonce；这是预期的竞争语义。
- `wait_for_all_submits=false` 优先返回最快提交结果；需要审计所有通道时才设为 `true`。
- durable nonce 延长交易有效期，但不会保持报价新鲜。签名前仍必须使用最新池状态。

## 6040 和有限重报价

收到 `BuySlippageBelowMinBaseAmountOut` 时：

1. 丢弃原交易和原 `min_base_amount_out`。
2. 获取比原快照更新的池状态和动态费率。
3. 检查事件/报价是否超过业务允许年龄。
4. 在风险上限内只重建并提交有限次数，例如一次。
5. 没有更新状态时不要用更大滑点重复发送同一报价。

失败交易中实际输出低于最低输出，说明保护生效；错误数量减少不能作为关闭保护的理由。

## 当前示例

| 示例 | 事件入口 | 交易路径 |
|---|---|---|
| `pumpfun_grpc_sniper` | `sol-parser-sdk` Yellowstone gRPC | 预热客户端 + blockhash cache + `Simple*Params` |
| `pumpfun_shredstream_sniper` | `sol-parser-sdk` ShredStream | create/首买上下文，无首次买入 RPC |
| `sol-trade-sdk/examples/pumpswap_trading` | `solana-streamer-sdk` gRPC | 动态费率事件参数；卖出前刷新池状态 |
| `pumpfun_trade` / `pumpswap_trade` | 无事件流 | RPC 驱动的完整买卖、SWQoS 和 nonce 配置 |

两个 `*_with_safekey` crate 使用 crates.io 的 `sol-safekey 0.1.8`，并与普通私钥循环示例统一使用 `sol-trade-sdk 4.0.22` 高层交易 API。keystore 解密应只在启动阶段完成，不能放入事件热路径。

生产部署还必须增加持仓隔离、最大损失、最大输入、目标 allowlist、签名持久化、重连去重、优雅停机和告警。
