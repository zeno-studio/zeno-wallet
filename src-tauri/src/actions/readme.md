┌─────────────┐
│  前端 UI    │
│ (Tauri Svelte)
│ amount, from_token, to_token
└─────┬───────┘
      │ invoke
      ▼
┌─────────────┐
│ Rust Command│
│ smart_swap()│
└─────┬───────┘
      │ 1. 查询缓存 / 2. 调用 RPC Provider
      ▼
┌────────────────────┐
│ DEX Aggregator RPC │
│ (Uniswap V2/V3,    │
│  1inch, Curve...)  │
└─────────┬──────────┘
          │ 返回价格 / 路径
          ▼
┌───────────────┐
│ 路径选择算法 │
│ (best output)│
└─────┬─────────┘
      │ 构建交易
      ▼
┌───────────────┐
│ RPC Provider   │
│ send_transaction│
└─────┬─────────┘
      ▼
┌───────────────┐
│ 更新本地历史  │
│ (DB / State)  │
└───────────────┘

关键设计点

缓存机制

对于同一交易对和金额，可以缓存 SwapQuote，避免每次重复计算

可扩展 TTL 或 moka 缓存

RPC Provider

已有的 RpcProvider 可封装调用 Uniswap V3 Router、1inch Aggregator、Curve Router 等

返回价格、路径、gas 估算

异步执行

所有查询都是 async，避免阻塞 Tauri 主线程

前端可以显示 loading 状态

本地状态更新

交易完成后，更新 AppState / DB，方便历史记录和 UI 展示

1️⃣ DEX Router 的核心函数

不同 DEX 的 Router 通常都有几个核心方法：

DEX	核心查询函数	功能
Uniswap V2	getAmountsOut(uint amountIn, address[] path)	给定输入 token 数量，返回沿路径的输出 token 数量
Uniswap V3	quoteExactInput(bytes path, uint amountIn)	给定输入数量，返回精确输出（考虑多跳池）
1inch Aggregator	getExpectedReturn 或通过 swap API 的 quote	返回最优路径和输出数量
Curve	get_dy(token_i, token_j, amount)	查询池中兑换输出

核心都是 “输入多少，输出多少”，不涉及签名和发送交易。

2️⃣ 智能路由 Swap 的流程

输入参数：from_token, to_token, amount_in, chain_id

轮询各个 DEX Router：

调用各 Router 提供的 quote / getAmountsOut / get_dy

获取对应输出数量、路径和可能手续费信息

选择最佳报价：

输出数量最大

可加入 gas 成本、滑点、手续费等加权

构建交易：

根据最佳路径生成 swap transaction

签名并发送：

用户在本地钱包签名，RPC Provider 发送

3️⃣ 注意事项

多跳交易：Uniswap V3 / 1inch 可以跨多个池，需要处理路径编码

不同链 DEX：各链 Router 地址不同，需要维护跨链映射

缓存报价：相同 token/amount 可以缓存几秒钟，减少重复调用

异步查询：轮询多个 DEX 可以并发执行，提高响应速度