核心思路
从 RPC 获取最新链信息（必要）：

当前区块头（baseFeePerGas, gas_limit, number, timestamp）。
当前建议的 gas_price / 费率：eth_gasPrice 或 eth_feeHistory。
交易相关账户状态（余额、nonce、code、storage 等）供 revm 仿真使用。
利用 revm crate 在本地执行交易模拟：

构造 EIP-1559 交易参数（max_fee_per_gas, max_priority_fee_per_gas 等）。
在 revm 中执行一次 “call” 模拟，得到 gas_used。
结合 base fee 和加成策略得到推荐 max_fee、max_priority 和 gas_limit。
采用二分搜索（或截断迭代）确定 gasLimit：

从最小值（如 21,000）和最大值（如当前 block gas limit 或 1.5 倍）开始。
每次用 revm 执行当前估算的 gas_limit 获得执行状态和 gas 消耗。
若模拟失败（out of gas/panic），调大 gas；反之调小，直到收敛。

