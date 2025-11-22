// src/rpc/public.rs

use crate::error::AppError;
use crate::state::{AppState, get_https_client};
use getrandom::getrandom;
use once_cell::sync::Lazy;
use reqwest::Client;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tauri::State;
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
pub struct PublicRpcNode {
    pub url: String,
    pub source: String,
}

static PUBLIC_NODES: Lazy<HashMap<&'static str, Vec<PublicRpcNode>>> = Lazy::new(|| {
    let mut m = HashMap::new();

    // Ethereum Mainnet
    m.insert("eth", vec![
        PublicRpcNode { url: "https://ethereum.publicnode.com".to_string(), source: "publicnode".into() },
        PublicRpcNode { url: "https://eth.merkle.io".to_string(), source: "merkle".into() },
        PublicRpcNode { url: "https://eth.llamarpc.com".to_string(), source: "llamarpc".into() },
        PublicRpcNode { url: "https://rpc.ankr.com/eth".to_string(), source: "ankr".into() },
    ]);

    // Base
    m.insert("base", vec![
        PublicRpcNode { url: "https://base.publicnode.com".to_string(), source: "publicnode".into() },
        PublicRpcNode { url: "https://base.meowrpc.com".to_string(), source: "meowrpc".into() },
        PublicRpcNode { url: "https://base.llamarpc.com".to_string(), source: "llamarpc".into() },
    ]);

    // BSC
    m.insert("bsc", vec![
        PublicRpcNode { url: "https://bsc.publicnode.com".to_string(), source: "publicnode".into() },
        PublicRpcNode { url: "https://bsc.meowrpc.com".to_string(), source: "meowrpc".into() },
    ]);

    // Linea
    m.insert("linea", vec![
        PublicRpcNode { url: "https://rpc.linea.build".to_string(), source: "official".into() },
        PublicRpcNode { url: "https://linea.decubate.com".to_string(), source: "decubate".into() },
    ]);

    // Polygon
    m.insert("polygon", vec![
        PublicRpcNode { url: "https://polygon-rpc.com".to_string(), source: "polygon-rpc".into() },
        PublicRpcNode { url: "https://polygon.llamarpc.com".to_string(), source: "llamarpc".into() },
    ]);

    // Arbitrum One
    m.insert("arbitrum", vec![
        PublicRpcNode { url: "https://arbitrum.meowrpc.com".to_string(), source: "meowrpc".into() },
        PublicRpcNode { url: "https://arbitrum.drpc.org".to_string(), source: "drpc".into() },
    ]);

    // 可继续添加：scroll, zksync, optimism, blast, mantle...
    m
});

#[derive(Clone)]
pub struct PublicRpc {
    client: Client,
    hosts: Vec<String>,
    current_idx: Arc<RwLock<usize>>,
    chain: String,
}

impl PublicRpc {
    /// 创建一个指定链的 PublicRpc 实例
    pub fn new(state: &State<AppState>, chain: &str) -> Result<Self, AppError> {
        let chain_key = chain.to_lowercase();
        let nodes = PUBLIC_NODES.get(chain_key.as_str())
            .ok_or(AppError::UnsupportedChain(chain.to_string()))?
            .clone();

        if nodes.is_empty() {
            return Err(AppError::NoAvailableRpcNodes(chain.to_string()));
        }

        let hosts: Vec<String> = nodes.into_iter().map(|n| n.url).collect();

        Ok(Self {
            client: get_https_client(state)?,
            hosts,
            current_idx: Arc::new(RwLock::new(0)),
            chain: chain_key,
        })
    }

    /// 随机打乱 hosts（启动时推荐调用一次）
    pub fn shuffle_hosts(&mut self) {
        use rand::seq::SliceRandom;
        use rand::thread_rng;
        let mut rng = thread_rng();
        self.hosts.shuffle(&mut rng);
    }

    // 核心：自动 fallback 调用
    async fn call_with_fallback(&self, method: &str, params: Value) -> Result<Value, AppError> {
        let mut last_err = None;

        for (i, host) in self.hosts.iter().enumerate() {
            match self.call_single(host, method, params.clone()).await {
                Ok(val) => {
                    *self.current_idx.write().await = i;
                    return Ok(val);
                }
                Err(e) => {
                    log::warn!("[{}] RPC {} failed: {} | {}", self.chain, host, method, e);
                    last_err = Some(e);
                }
            }
        }

        Err(last_err.unwrap_or(AppError::AllPublicRpcFailed(self.chain.clone())))
    }

    async fn call_single(&self, url: &str, method: &str, params: Value) -> Result<Value, AppError> {
        let payload = json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
            "id": 1
        });

        let resp = self.client
            .post(url)
            .timeout(Duration::from_secs(15))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(AppError::ReqwestError)?;

        let json: Value = resp.json().await.map_err(AppError::JsonParseError)?;

        if let Some(err) = json.get("error") {
            let code = err["code"].as_i64().unwrap_or(0);
            let msg = err["message"].as_str().unwrap_or("unknown error").to_string();
            return Err(AppError::RpcError(code as u64, msg));
        }

        Ok(json["result"].clone())
    }

    // ==================== 常用方法封装 ====================
  pub async fn block_number(&self) -> Result<u64, AppError> {
        let result = self.call_with_fallback("eth_blockNumber", json!([])).await?;
        let hex: String = serde_json::from_value(result)?;
        Ok(u64::from_str_radix(hex.trim_start_matches("0x"), 16)?)
    }

    pub async fn get_balance(&self, address: &str, block: &str) -> Result<String, AppError> {
        let result = self.call_with_fallback("eth_getBalance", json!([address, block])).await?;
        Ok(result.as_str().unwrap_or("0x0").to_string())
    }

    pub async fn get_nonce(&self, address: &str, block: &str) -> Result<u64, AppError> {
        let result = self.call_with_fallback("eth_getTransactionCount", json!([address, block])).await?;
        let hex: String = serde_json::from_value(result)?;
        Ok(u64::from_str_radix(hex.trim_start_matches("0x"), 16)?)
    }

    pub async fn gas_price(&self) -> Result<String, AppError> {
        let result = self.call_with_fallback("eth_gasPrice", json!([])).await?;
        Ok(result.as_str().unwrap_or("0x0").to_string())
    }

    pub async fn max_priority_fee(&self) -> Result<String, AppError> {
        let result = self.call_with_fallback("eth_maxPriorityFeePerGas", json!([])).await?;
        Ok(result.as_str().unwrap_or("0x0").to_string())
    }

    pub async fn send_raw_transaction(&self, signed_tx: &str) -> Result<String, AppError> {
        let result = self.call_with_fallback("eth_sendRawTransaction", json!([signed_tx])).await?;
        Ok(result.as_str().unwrap_or("").to_string())
    }

    pub async fn get_transaction_receipt(&self, hash: &str) -> Result<Option<Value>, AppError> {
        let result = self.call_with_fallback("eth_getTransactionReceipt", json!([hash])).await?;
        if result.is_null() {
            Ok(None)
        } else {
            Ok(Some(result))
        }
    }

    pub async fn get_transaction_by_hash(&self, hash: &str) -> Result<Option<Value>, AppError> {
        let result = self.call_with_fallback("eth_getTransactionByHash", json!([hash])).await?;
        if result.is_null() {
            Ok(None)
        } else {
            Ok(Some(result))
        }
    }

    pub async fn chain_id(&self) -> Result<u64, AppError> {
        let result = self.call_with_fallback("eth_chainId", json!([])).await?;
        let hex: String = serde_json::from_value(result)?;
        Ok(u64::from_str_radix(hex.trim_start_matches("0x"), 16)?)
    }

    pub async fn eth_call(&self, to: &str, data: &str, block: &str) -> Result<String, AppError> {
        let result = self.call_with_fallback("eth_call", json!([
            {"to": to, "data": data},
            block
        ])).await?;
        Ok(result.as_str().unwrap_or("0x").to_string())
    }

    // 继续加你需要的：eth_call, gas_price, estimate_gas ...
}