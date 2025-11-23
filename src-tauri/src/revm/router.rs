pub async fn estimate_gas_smart(
    &self,
    from: Address,
    to: Option<Address>,
    value: U256,
    data: Bytes,
) -> eyre::Result<GasEstimateResult> {
    // ==================== 1. 快路径：原生转账 & 简单 ERC20 ====================
    if data.is_empty() {
        if value > U256::ZERO {
            return Ok(self.native_transfer_gas()); // 21_000
        }
    }

    if let Some(to_addr) = to {
        // ERC20 transfer / transferFrom 签名
        if data.len() == 68 && &data[0..4] == ERC20_TRANSFER_SIG {
            return Ok(self.erc20_transfer_gas()); // ~65_000
        }
        if data.len() == 68 && &data[0..4] == ERC20_APPROVE_SIG {
            return Ok(self.erc20_approve_gas());   // ~46_000
        }
    }

    // ==================== 2. DeFi 复杂交易主路径 ====================
    // 先尝试 eth_estimateGas（2025 年节点对 Proxy 支持已非常好）
    if let Ok(gas) = self.try_eth_estimate_gas(from, to, value, &data).await {
        if gas >= 100_000 {
            return Ok(self.with_buffer(gas, 1.5)); // DeFi 统一 50% buffer
        }
    }

    // ==================== 3. 失败 → Alchemy 模拟节点（终极准星）===================
    if let Ok(gas) = self.simulate_with_alchemy(from, to, value, &data).await {
        return Ok(self.with_buffer(gas, 1.25));
    }

    // ==================== 4. 最终保命：硬编码表 ====================
    let gas_limit = self.get_hardcoded_limit(to, &data);
    
    Ok(GasEstimateResult {
        gas_limit,
        simulation_gas_used: gas_limit,
        ..self.build_fee_fields().await?
    })
}

static ERC20_TRANSFER_SIG: [u8; 4] = [0xa9, 0x05, 0x9c, 0xbb];
static ERC20_APPROVE_SIG: [u8; 4] = [0x09, 0x5e, 0xa7, 0xb3];

impl Wallet {
    fn native_transfer_gas(&self) -> GasEstimateResult {
        GasEstimateResult { gas_limit: 21_000, simulation_gas_used: 21_000, .. }
    }
    fn erc20_transfer_gas(&self) -> GasEstimateResult {
        GasEstimateResult { gas_limit: 75_000, simulation_gas_used: 65_000, .. }
    }
    fn with_buffer(&self, gas: u64, multiplier: f64) -> GasEstimateResult {
        let limit = (gas as f64 * multiplier).ceil() as u64;
        GasEstimateResult { gas_limit: limit.min(30_000_000), simulation_gas_used: gas, .. }
    }
}