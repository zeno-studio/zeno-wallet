// src-tauri/src/rpc/request.rs
use crate::error::AppError;
use crate::utils::num::str_to_u256;
use alloy_primitives::U256;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RpcMethod {
    // ====== 轻客户端必须原生支持的核心方法（Helios 能 100% 高效处理）======
    EthChainId,
    EthBlockNumber,
    EthGetBalance,
    EthGetTransactionCount,
    EthGetBlockByNumber,
    EthGetBlockByHash,
    EthGetTransactionByHash,
    EthGetTransactionReceipt,  // 加这个！dApp 经常查交易状态
    EthGasPrice,
    EthMaxPriorityFeePerGas,   // EIP-1559 必备
    EthFeeHistory,             // wagmi 用这个算 fee
    EthCall,
    EthEstimateGas,
    EthSendRawTransaction,

    // ====== 轻量但高频的（Helios 支持）======
    EthGetCode,
    EthGetStorageAt,

    // ====== 客户端信息（固定返回）======
    Web3ClientVersion,
    NetVersion,
    EthSyncing,

    // ====== 所有其他方法全部走 Custom（强烈推荐！）======
    Custom(&'static str),
}
impl RpcMethod {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::EthChainId => "eth_chainId",
            Self::EthBlockNumber => "eth_blockNumber",
            Self::EthGetBalance => "eth_getBalance",
            Self::EthGetTransactionCount => "eth_getTransactionCount",
            Self::EthGetBlockByNumber => "eth_getBlockByNumber",
            Self::EthGetBlockByHash => "eth_getBlockByHash",
            Self::EthGetTransactionByHash => "eth_getTransactionByHash",
            Self::EthGetTransactionReceipt => "eth_getTransactionReceipt", // 加这个
            Self::EthGasPrice => "eth_gasPrice",
            Self::EthMaxPriorityFeePerGas => "eth_maxPriorityFeePerGas",
            Self::EthFeeHistory => "eth_feeHistory",
            Self::EthCall => "eth_call",
            Self::EthEstimateGas => "eth_estimateGas",
            Self::EthSendRawTransaction => "eth_sendRawTransaction",
            Self::EthGetCode => "eth_getCode",
            Self::EthGetStorageAt => "eth_getStorageAt",
            Self::Web3ClientVersion => "web3_clientVersion",
            Self::NetVersion => "net_version",
            Self::EthSyncing => "eth_syncing",
            Self::Custom(s) => s,
        }
    }
}

static GLOBAL_JSONRPC_ID: AtomicU64 = AtomicU64::new(1);

fn next_id() -> u64 {
    GLOBAL_JSONRPC_ID.fetch_add(1, Ordering::Relaxed)
}

/// 通用的 JSON-RPC 请求体
#[derive(Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub method: String,
    pub params: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Value>, // 允许为 null（某些节点只认 null）
}

impl JsonRpcRequest {
    pub fn new(method: impl Into<String>, params: Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            method: method.into(),
            params,
            id: Some(json!(next_id())), // 自动递增
        }
    }

    // 给批量请求用的 null id 版本
    pub fn new_batch(method: impl Into<String>, params: Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            method: method.into(),
            params,
            id: None,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: Option<String>,
    pub id: Option<Value>,
    pub result: Option<Value>,
    pub error: Option<RpcError>,
}

#[derive(Serialize, Deserialize)]
pub struct RpcError {
    pub code: i64,
    pub message: String,
}

/// 通用调用（推荐用于自定义方法）
pub async fn call(client: Client, url: &str, request: JsonRpcRequest) -> Result<Value, AppError> {
    let payload = json!(request);
    let response = client
        .post(url)
        .json(&payload)
        .send()
        .await
        .map_err(AppError::ReqwestClientConnectionError)?;

    let json: JsonRpcResponse = response.json().await.map_err(AppError::JsonParseError)?;

    // 验证响应格式
    if json.jsonrpc.as_deref() != Some("2.0") {
        return Err(AppError::JsonRpcInvalidResponse);
    }

    // 检查ID是否匹配
    if let (Some(req_id), Some(resp_id)) = (request.id.to_string(), json.id.clone()) {
        if req_id != resp_id.to_string() {
            return Err(AppError::JsonRpcInvalidId);
        }
    }

    // 处理错误响应
    if let Some(err) = json.error {
        return Err(AppError::HttpsRpcError(err.code as u64, err.message));
    }

    // 返回结果
    json.result.ok_or(AppError::JsonRpcMissingResult)
}

// ==================== 常用方法封装 ====================

pub async fn block_number(client: Client, url: &str) -> Result<u64, AppError> {
    let req = JsonRpcRequest::new(RpcMethod::EthBlockNumber.as_str(), json!([]));
    let result = call(client, url, req).await?;
    let hex: String = serde_json::from_value(result).map_err(|e| AppError::JsonParseError(e))?;
    Ok(u64::from_str_radix(&hex.trim_start_matches("0x"), 16)
        .map_err(|_| AppError::HttpsRpcError(0, "Invalid hex block number".into()))?)
}

pub async fn get_balance(
    client: Client,
    url: &str,
    address: &str,
    block: &str,
) -> Result<String, AppError> {
    let req = JsonRpcRequest::new(RpcMethod::EthGetBalance.as_str(), json!([address, block]));
    let result = call(client, url, req).await?;
    Ok(result.as_str().unwrap_or_default().to_string())
}

pub async fn get_nonce(
    client: Client,
    url: &str,
    address: &str,
    block: &str,
) -> Result<u64, AppError> {
    let req = JsonRpcRequest::new(
        RpcMethod::EthGetTransactionCount.as_str(),
        json!([address, block]),
    );
    let result = call(client, url, req).await?;
    let hex: String = serde_json::from_value(result).map_err(|e| AppError::JsonParseError(e))?;
    Ok(u64::from_str_radix(&hex.trim_start_matches("0x"), 16)
        .map_err(|_| AppError::NumberParseError))
}

pub async fn gas_price(client: Client, url: &str) -> Result<U256, AppError> {
    let req = JsonRpcRequest::new(RpcMethod::EthGasPrice.as_str(), json!([]));
    let result = call(client, url, req).await?;
    let hex: String = serde_json::from_value(result).map_err(|e| AppError::JsonParseError(e))?;
    Ok(str_to_u256(&hex)).map_err(|_| AppError::NumberParseError)
}

pub async fn max_priority_fee(client: Client, url: &str) -> Result<U256, AppError> {
    let req = JsonRpcRequest::new(
        RpcMethod::EthMaxPriorityFeePerGas.as_str(),
        json!([]),
    );
    let result = call(client, url, req).await?;
    let hex: String = serde_json::from_value(result).map_err(|e| AppError::JsonParseError(e))?;
    Ok(str_to_u256(&hex)).map_err(|_| AppError::NumberParseError)
}

pub async fn fee_history(
    client: Client,
    url: &str,
    block_count: u64,
    reward_percentiles: Vec<f64>,
) -> Result<Value, AppError> {
    let req = JsonRpcRequest::new(
        RpcMethod::EthFeeHistory.as_str(),
        json!([block_count, "latest", reward_percentiles]),
    );
    let result = call(client, url, req).await?;
    Ok(result)
}

pub async fn get_block_by_hash(
    client: Client,
    url: &str,
    block_hash: &str,
    full_tx: bool,
) -> Result<Value, AppError> {
    let req = JsonRpcRequest::new(
        RpcMethod::EthGetBlockByHash.as_str(),
        json!([block_hash, full_tx]),
    );
    call(client, url, req).await
}

pub async fn get_transaction_by_hash(
    client: Client,
    url: &str,
    tx_hash: &str,
) -> Result<Value, AppError> {
    let req = JsonRpcRequest::new(
        RpcMethod::EthGetTransactionByHash.as_str(),
        json!([tx_hash]),
    );
    call(client, url, req).await
}

pub async fn get_transaction_receipt(
    client: Client,
    url: &str,
    tx_hash: &str,
) -> Result<Value, AppError> {
    let req = JsonRpcRequest::new(
        RpcMethod::EthGetTransactionReceipt.as_str(),
        json!([tx_hash]),
    );
    call(client, url, req).await
}

pub async fn eth_call(
    client: Client,
    url: &str,
    to: &str,
    data: &str,
    block: &str,
) -> Result<Value, AppError> {
    let req = JsonRpcRequest::new(
        RpcMethod::EthCall.as_str(),
        json!([{"to": to, "data": data}, block]),
    );
    call(client, url, req).await
}

pub async fn estimate_gas(
    client: Client,
    url: &str,
    to: &str,
    data: &str,
) -> Result<U256, AppError> {
    let req = JsonRpcRequest::new(
        RpcMethod::EthEstimateGas.as_str(),
        json!([{"to": to, "data": data}]),
    );
    let result = call(client, url, req).await?;
    let hex: String = serde_json::from_value(result).map_err(|e| AppError::JsonParseError(e))?;
    Ok(str_to_u256(&hex)).map_err(|_| AppError::NumberParseError)
}

pub async fn get_code(
    client: Client,
    url: &str,
    address: &str,
    block: &str,
) -> Result<String, AppError> {
    let req = JsonRpcRequest::new(
        RpcMethod::EthGetCode.as_str(),
        json!([address, block]),
    );
    let result = call(client, url, req).await?;
    Ok(result.as_str().unwrap_or_default().to_string())
}

pub async fn get_storage_at(
    client: Client,
    url: &str,
    address: &str,
    position: &str,
    block: &str,
) -> Result<String, AppError> {
    let req = JsonRpcRequest::new(
        RpcMethod::EthGetStorageAt.as_str(),
        json!([address, position, block]),
    );
    let result = call(client, url, req).await?;
    Ok(result.as_str().unwrap_or_default().to_string())
}

pub async fn web3_client_version(client: Client, url: &str) -> Result<String, AppError> {
    let req = JsonRpcRequest::new(RpcMethod::Web3ClientVersion.as_str(), json!([]));
    let result = call(client, url, req).await?;
    Ok(result.as_str().unwrap_or_default().to_string())
}

pub async fn net_version(client: Client, url: &str) -> Result<String, AppError> {
    let req = JsonRpcRequest::new(RpcMethod::NetVersion.as_str(), json!([]));
    let result = call(client, url, req).await?;
    Ok(result.as_str().unwrap_or_default().to_string())
}

pub async fn eth_syncing(client: Client, url: &str) -> Result<bool, AppError> {
    let req = JsonRpcRequest::new(RpcMethod::EthSyncing.as_str(), json!([]));
    let result = call(client, url, req).await?;
    Ok(result.as_bool().unwrap_or(false))
}

pub async fn chain_id(client: Client, url: &str) -> Result<u64, AppError> {
    let req = JsonRpcRequest::new(RpcMethod::EthChainId.as_str(), json!([]));
    let result = call(client, url, req).await?;
    let hex: String = serde_json::from_value(result).map_err(|e| AppError::JsonParseError(e))?;
    Ok(u64::from_str_radix(&hex.trim_start_matches("0x"), 16)
        .map_err(|_| AppError::NumberParseError))
}

pub async fn send_raw_transaction(
    client: Client,
    url: &str,
    signed_tx: &str,
) -> Result<String, AppError> {
    let req = JsonRpcRequest::new(
        RpcMethod::EthSendRawTransaction.as_str(),
        json!([signed_tx]),
    );
    let result = call(client, url, req).await?;
    Ok(result.as_str().unwrap_or_default().to_string())
}

// ==================== Tauri Command 示例（可选）================

#[tauri::command]
pub async fn get_eth_balance(rpc_url: String, address: String) -> Result<String, String> {
    let client = Client::new(); // 创建一个新的客户端实例
    let balance = get_balance(client, &rpc_url, &address, "latest")
        .await
        .map_err(|e| e.to_string())?;
    Ok(balance)
}
