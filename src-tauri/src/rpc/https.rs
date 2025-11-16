// src-tauri/src/rpc/https.rs
use crate::error::AppError;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RpcMethod {
    EthBlockNumber,
    EthGetBalance,
    EthGetTransactionCount,
    EthGasPrice,
    EthMaxPriorityFeePerGas,
    EthFeeHistory,
    EthChainId,
    EthSendRawTransaction,
    EthGetTransactionByHash,
    EthGetBlockByNumber,
    Custom(&'static str), // 自定义方法
}

impl RpcMethod {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::EthBlockNumber => "eth_blockNumber",
            Self::EthGetBalance => "eth_getBalance",
            Self::EthGetTransactionCount => "eth_getTransactionCount",
            Self::EthGasPrice => "eth_gasPrice",
            Self::EthMaxPriorityFeePerGas => "eth_maxPriorityFeePerGas",
            Self::EthFeeHistory => "eth_feeHistory",
            Self::EthChainId => "eth_chainId",
            Self::EthSendRawTransaction => "eth_sendRawTransaction",
            Self::EthGetTransactionByHash => "eth_getTransactionByHash",
            Self::EthGetBlockByNumber => "eth_getBlockByNumber",
            Self::Custom(s) => s,
        }
    }
}

/// 通用的 JSON-RPC 请求体
#[derive(Serialize)]
struct JsonRpcRequest {
    jsonrpc: &'static str,
    id: u64,
    method: String,
    params: Value,
}

#[derive(Deserialize)]
struct JsonRpcResponse {
    result: Option<Value>,
    error: Option<RpcError>,
}

#[derive(Deserialize)]
struct RpcError {
    code: i64,
    message: String,
}

/// 通用 Ethereum JSON-RPC Provider
pub struct EthRpcProvider {
    client: Client,
    url: String,
    next_id: std::sync::atomic::AtomicU64,
}

impl EthRpcProvider {
    /// 创建一个高性能的 RPC Provider
    pub fn new(url: &str) -> Result<Self, AppError> {
        let client = Client::builder()
            .use_rustls_tls()
            .pool_max_idle_per_host(10)
            .http2_keep_alive_timeout(Duration::from_secs(30))
            .timeout(Duration::from_secs(10))
            .gzip(true)
            .brotli(true)
            .build()
            .map_err(|e| {
                AppError::HttpsRpcError(format!("Failed to build reqwest client: {}", e))
            })?;

        Ok(Self {
            client,
            url: url.to_string(),
            next_id: std::sync::atomic::AtomicU64::new(1),
        })
    }

    /// 通用调用（推荐用于自定义方法）
    pub async fn call(&self, method: RpcMethod, params: impl Serialize) -> Result<Value, AppError> {
        let id = self
            .next_id
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let payload = json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method.as_str(),
            "params": params
        });

        let response = self
            .client
            .post(&self.url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| AppError::HttpsRpcError(format!("Request failed: {}", e)))?;

        let json: JsonRpcResponse = response
            .json()
            .await
            .map_err(|e| AppError::HttpsRpcError(format!("Invalid JSON response: {}", e)))?;

        if let Some(err) = json.error {
            return Err(AppError::HttpsRpcError(format!(
                "RPC Error {}: {}",
                err.code, err.message
            )));
        }

        json.result
            .ok_or_else(|| AppError::HttpsRpcError("No result in response".into()))
    }

    // ==================== 常用方法封装 ====================

    pub async fn block_number(&self) -> Result<u64, AppError> {
        let result = self.call(RpcMethod::EthBlockNumber, json!([])).await?;
        let hex: String = serde_json::from_value(result)
            .map_err(|_| AppError::HttpsRpcError("Failed to parse block number".into()))?;
        Ok(u64::from_str_radix(&hex.trim_start_matches("0x"), 16)
            .map_err(|_| AppError::HttpsRpcError("Invalid hex block number".into()))?)
    }

    pub async fn get_balance(&self, address: &str, block: &str) -> Result<String, AppError> {
        self.call(RpcMethod::EthGetBalance, json!([address, block]))
            .await
            .map(|v| v.as_str().unwrap_or_default().to_string())
    }

    pub async fn get_nonce(&self, address: &str, block: &str) -> Result<u64, AppError> {
        let result = self
            .call(RpcMethod::EthGetTransactionCount, json!([address, block]))
            .await?;
        let hex: String = serde_json::from_value(result)
            .map_err(|_| AppError::HttpsRpcError("Failed to parse nonce".into()))?;
        Ok(u64::from_str_radix(&hex.trim_start_matches("0x"), 16)
            .map_err(|_| AppError::HttpsRpcError("Invalid hex nonce".into()))?)
    }

    pub async fn gas_price(&self) -> Result<u128, AppError> {
        let result = self.call(RpcMethod::EthGasPrice, json!([])).await?;
        let hex: String = serde_json::from_value(result)
            .map_err(|_| AppError::HttpsRpcError("Failed to parse gas price".into()))?;
        Ok(u128::from_str_radix(&hex.trim_start_matches("0x"), 16)
            .map_err(|_| AppError::HttpsRpcError("Invalid hex gas price".into()))?)
    }

    pub async fn max_priority_fee(&self) -> Result<u128, AppError> {
        let result = self
            .call(RpcMethod::EthMaxPriorityFeePerGas, json!([]))
            .await?;
        let hex: String = serde_json::from_value(result)
            .map_err(|_| AppError::HttpsRpcError("Failed to parse priority fee".into()))?;
        Ok(u128::from_str_radix(&hex.trim_start_matches("0x"), 16)
            .map_err(|_| AppError::HttpsRpcError("Invalid hex priority fee".into()))?)
    }

    pub async fn chain_id(&self) -> Result<u64, AppError> {
        let result = self.call(RpcMethod::EthChainId, json!([])).await?;
        let hex: String = serde_json::from_value(result)
            .map_err(|_| AppError::HttpsRpcError("Failed to parse chain ID".into()))?;
        Ok(u64::from_str_radix(&hex.trim_start_matches("0x"), 16)
            .map_err(|_| AppError::HttpsRpcError("Invalid hex chain ID".into()))?)
    }

    pub async fn send_raw_transaction(&self, signed_tx: &str) -> Result<String, AppError> {
        self.call(RpcMethod::EthSendRawTransaction, json!([signed_tx]))
            .await
            .map(|v| v.as_str().unwrap_or_default().to_string())
    }
}

// ==================== Tauri Command 示例（可选）================

#[tauri::command]
pub async fn get_eth_balance(rpc_url: String, address: String) -> Result<String, String> {
    let provider = EthRpcProvider::new(&rpc_url).map_err(|e| e.to_string())?;
    let balance = provider
        .get_balance(&address, "latest")
        .await
        .map_err(|e| e.to_string())?;
    Ok(balance)
}