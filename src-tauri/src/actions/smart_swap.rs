use std::collections::HashMap;
use ethers::prelude::*;
use crate::rpc::RpcProvider;
use crate::state::AppState;

#[derive(Debug, Clone)]
pub struct SwapQuote {
    pub from_token: Address,
    pub to_token: Address,
    pub amount_in: U256,
    pub amount_out: U256,
    pub dex: String,
    pub path: Vec<Address>,
    pub gas_estimate: U256,
}

pub struct SmartRouter {
    pub rpc: RpcProvider,
    pub cache: HashMap<(Address, Address, U256), SwapQuote>,
}

impl SmartRouter {
    pub fn new(rpc: RpcProvider) -> Self {
        Self {
            rpc,
            cache: HashMap::new(),
        }
    }

    /// 查询最佳 swap 路径
    pub async fn get_best_quote(
        &mut self,
        from_token: Address,
        to_token: Address,
        amount_in: U256,
        chain_id: u64,
    ) -> anyhow::Result<SwapQuote> {
        // 1️⃣ 检查缓存
        if let Some(q) = self.cache.get(&(from_token, to_token, amount_in)) {
            return Ok(q.clone());
        }

        // 2️⃣ 查询不同 DEX
        let mut quotes: Vec<SwapQuote> = Vec::new();

        // Uniswap V3
        if let Ok(q) = self.rpc.query_uniswap_v3(from_token, to_token, amount_in, chain_id).await {
            quotes.push(q);
        }

        // 1inch
        if let Ok(q) = self.rpc.query_1inch(from_token, to_token, amount_in, chain_id).await {
            quotes.push(q);
        }

        // Curve / 其他 DEX
        // ...

        // 3️⃣ 选择最佳 output
        let best = quotes.iter().max_by_key(|q| q.amount_out).ok_or_else(|| anyhow::anyhow!("No swap path"))?.clone();

        // 4️⃣ 缓存结果
        self.cache.insert((from_token, to_token, amount_in), best.clone());

        Ok(best)
    }

    /// 构建并发送交易
    pub async fn execute_swap(
        &self,
        quote: SwapQuote,
        user_wallet: &LocalWallet,
    ) -> anyhow::Result<TxHash> {
        let tx = self.rpc.build_swap_transaction(&quote).await?;
        let pending_tx = user_wallet.send_transaction(tx, None).await?;
        Ok(*pending_tx)
    }
}
