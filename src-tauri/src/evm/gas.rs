
use alloy::network::{Ethereum, TransactionBuilder};
use alloy::rpc::primitives::{U128, U256};
use alloy::rpc::types::eth::{BlockId, FeeHistory};
use std::time::Duration;
use tauri::async_runtime;



/// 2025 年最推荐的参数
const FEE_HISTORY_BLOCKS: u64 = 20;           // 看最近 20 个块
const REWARD_PERCENTILES: [f64; 3] = [10.0, 50.0, 90.0]; // 保守、中等、激进

pub struct PriorityFeeEstimator {
    provider: AlloyProvider,
}

impl PriorityFeeEstimator {
    pub fn new(rpc_url: &str) -> Self {
        let provider = ProviderBuilder::new()
            .on_http(rpc_url.parse().expect("invalid rpc url"));
        Self { provider }
    }

    /// 返回 (base_fee_per_gas 当前, 推荐的 max_priority_fee_per_gas)
    /// 第三个返回值是预估的下一块 base_fee（大多数 UI 都直接显示这个）
    pub async fn estimate_priority_fees(&self) -> Result<(U256, U256, U256), alloy::providers::ProviderError> {
        // 1. 先拿最新块，获取当前 baseFeePerGas
        let latest_block = self.provider.get_block_by_number(BlockId::latest(), false).await?
            .expect("latest block not found");
        let current_base_fee = latest_block.header.base_fee_per_gas
            .unwrap_or_default(); // 老链可能没有，默认为 0

        // 2. 拿 fee history 计算优先费
        let fee_history: FeeHistory = self.provider
            .get_fee_history(
                FEE_HISTORY_BLOCKS,
                BlockId::latest(),
                &REWARD_PERCENTILES,
            )
            .await?;

        // reward[i][0] = 第 i 个块的第 10 百分位 priority fee
        // reward[i][1] = 第 50 百分位
        // reward[i][2] = 第 90 百分位
        let mut percentiles = fee_history.reward.unwrap_or_default();

        // 取最近 5 个块的中位数作为推荐值（最稳健）
        let recent_rewards: Vec<U256> = percentiles
            .iter()
            .rev()
            .take(5)
            .map(|rewards| rewards[1]) // 中位数
            .collect();

        let mut recommended_priority = if recent_rewards.is_empty() {
            U256::from(1_000_000_000u64) // 1 gwei 保底
        } else {
            let sum: U256 = recent_rewards.iter().sum();
            sum / U256::from(recent_rewards.len())
        };

        // 防止极端情况太低（比如 L2 突然 0 fee）
        if recommended_priority < U256::from(100_000_000u64) {
            recommended_priority = U256::from(100_000_000u64); // 0.1 gwei 最低
        }

        // 3. 计算下一块预计 base fee（EIP-1559 公式）
        let next_base_fee = estimate_next_base_fee(&latest_block, &fee_history);

        Ok((current_base_fee, recommended_priority, next_base_fee))
    }
}

/// 精确实现 EIP-1559 下一块 base fee 预估
fn estimate_next_base_fee(
    latest_block: &alloy::rpc::types::eth::Block,
    fee_history: &FeeHistory,
) -> U256 {
    let base_fee = latest_block.header.base_fee_per_gas.unwrap_or_default();
    let gas_used = latest_block.header.gas_used;
    let gas_limit = latest_block.header.gas_limit;

    let gas_used_ratio = if gas_limit == 0 {
        U256::ZERO
    } else {
        U256::from(gas_used) * 100) / U256::from(gas_limit)
    };

    // EIP-1559 目标是 50% 填充率
    let delta = if gas_used_ratio > U256::from(50) {
        base_fee * (gas_used_ratio - U256::from(50)) / U256::from(50) / U256::from(8)
    } else {
        base_fee * (U256::from(50) - gas_used_ratio) / U256::from(50) / U256::from(8)
    };

    if gas_used_ratio > U256::from(50) {
        base_fee + delta
    } else {
        base_fee.saturating_sub(delta)
    }
}
