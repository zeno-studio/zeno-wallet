// src/helios_protocol/handler.rs
use crate::core::state::AppState;
use crate::error::AppError;
use crate::rpc::method::{JsonRpcRequest, JsonRpcResponse, RpcMethod, RpcError};
use crate::utils::num::str_to_u256;

use alloy_primitives::{Address, TxHash, U256};
use helios::client::HeliosClient;
use helios::database::FileDB;
use helios::types::{BlockTag, CallRequest};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use tauri::{AppHandle, Runtime};
use tokio::sync::Mutex;

/// 单个或批量请求
#[derive(serde::Deserialize)]
#[serde(untagged)]
pub enum JsonRpcRequestOrBatch {
    Single(JsonRpcRequest),
    Batch(Vec<JsonRpcRequest>),
}

/// 单个或批量响应
#[derive(serde::Serialize)]
#[serde(untagged)]
pub enum JsonRpcResponseOrBatch {
    Single(JsonRpcResponse),
    Batch(Vec<JsonRpcResponse>),
}

/// 主处理器 —— lib.rs 只需一行注册
pub fn helios_protocol_handler<R: Runtime>(
    app: AppHandle<R>,
    request: tauri::http::Request<Vec<u8>>,
) -> tauri::http::Response<Vec<u8>> {
    // 只接受 POST
    if request.method() != "POST" {
        return tauri::http::ResponseBuilder::new()
            .status(405)
            .body(b"Only POST allowed".to_vec())
            .unwrap();
    }

    let body = match std::str::from_utf8(&request.body()) {
        Ok(b) => b,
        Err(_) => {
            return tauri::http::ResponseBuilder::new()
                .status(400)
                .body(br#"{"jsonrpc":"2.0","error":{"code":-32700,"message":"Invalid UTF-8"},"id":null}"#.to_vec())
                .unwrap();
        }
    };

    // 解析为我们自己的类型
    let payload: JsonRpcRequestOrBatch = match serde_json::from_str(body) {
        Ok(p) => p,
        Err(_) => {
            return tauri::http::ResponseBuilder::new()
                .status(400)
                .body(br#"{"jsonrpc":"2.0","error":{"code":-32700,"message":"Parse error"},"id":null}"#.to_vec())
                .unwrap();
        }
    };

    // 取出 state
    let state = app.state::<AppState>();

    // 异步处理
    let response = tauri::async_runtime::block_on(async {
        match payload {
            JsonRpcRequestOrBatch::Single(req) => {
                JsonRpcResponseOrBatch::Single(process_one(&state, &req).await)
            }
            JsonRpcRequestOrBatch::Batch(reqs) => {
                let results = futures::future::join_all(
                    reqs.into_iter().map(|req| process_one(&state, &req))
                ).await;
                JsonRpcResponseOrBatch::Batch(results)
            }
        }
    });

    // 序列化返回
    match serde_json::to_vec(&response) {
        Ok(bytes) => tauri::http::ResponseBuilder::new()
            .header("Content-Type", "application/json")
            .header("Access-Control-Allow-Origin", "*")
            .header("Cache-Control", "no-cache")
            .body(bytes)
            .unwrap(),
        Err(_) => tauri::http::ResponseBuilder::new()
            .status(500)
            .body(br#"{"jsonrpc":"2.0","error":{"code":-32603,"message":"Internal serialization error"},"id":null}"#.to_vec())
            .unwrap(),
    }
}

/// 核心处理单个请求（支持多链）
async fn process_one(
    state: &AppState,
    req: &JsonRpcRequest,
) -> JsonRpcResponse {
    // 1. 获取当前链 + 对应 Helios 实例
    let current_chain = state.current_chain.lock().await;
    let clients = state.helios_clients.lock().await;
    let client = match clients.get(&*current_chain) {
        Some(c) => c,
        None => {
            return JsonRpcResponse {
                jsonrpc: Some("2.0".to_string()),
                id: req.id.map(|v| json!(v)),
                result: None,
                error: Some(crate::rpc::request::RpcError {
                    code: -32000,
                    message: "Helios client not initialized for current chain".to_string(),
                }),
            };
        }
    };

    let method = str_to_method(&req.method);

    let result: Result<Value, AppError> = match method {
        RpcMethod::EthChainId => client.get_chain_id().await.map(|id| json!(format!("0x{id:x}"))),
        RpcMethod::EthBlockNumber => client.get_block_number().await.map(|n| json!(format!("0x{n:x}"))),
        RpcMethod::EthGetBalance => {
            let addr: Address = parse_param(&req.params, 0)?;
            let tag = parse_block_tag(&req.params, 1)?;
            client.get_balance(&addr, tag).await.map(|b| json!(b.to_string()))
        }
        RpcMethod::EthGetTransactionCount => {
            let addr: Address = parse_param(&req.params, 0)?;
            let tag = parse_block_tag(&req.params, 1)?;
            client.get_nonce(&addr, tag).await.map(|n| json!(format!("0x{n:x}")))
        }
        RpcMethod::EthGasPrice => client.get_gas_price().await.map(|p| json!(p.to_string())),
        RpcMethod::EthMaxPriorityFeePerGas => {
            // 使用 gas price 作为优先费用的近似值
            client.get_gas_price().await.map(|p| json!(p.to_string()))
        },
        RpcMethod::EthFeeHistory => {
            // 简化处理，实际实现可能需要更多参数
            client.get_gas_price().await.map(|p| json!(p.to_string())) // 临时使用 gas price 作为替代
        }
        RpcMethod::EthSendRawTransaction => {
            let raw: String = parse_param(&req.params, 0)?;
            client.send_raw_transaction(&raw).await.map(|h| json!(h.to_string()))
        }
        RpcMethod::EthGetBlockByNumber => {
            let tag = parse_block_tag(&req.params, 0)?;
            let full = req.params.as_array().and_then(|a| a.get(1)).and_then(|v| v.as_bool()).unwrap_or(false);
            client.get_block_by_number(tag, full).await.map(|b| json!(b))
        }
        RpcMethod::EthGetBlockByHash => {
            let hash: TxHash = parse_param(&req.params, 0)?;
            let full = req.params.as_array().and_then(|a| a.get(1)).and_then(|v| v.as_bool()).unwrap_or(false);
            client.get_block_by_hash(&hash, full).await.map(|b| json!(b))
        }
        RpcMethod::EthGetTransactionByHash => {
            let hash: TxHash = parse_param(&req.params, 0)?;
            client.get_transaction_by_hash(&hash).await.map(|t| json!(t))
        }
        RpcMethod::EthGetTransactionReceipt => {
            let hash: TxHash = parse_param(&req.params, 0)?;
            client.get_transaction_receipt(&hash).await.map(|r| json!(r))
        }
        RpcMethod::EthCall => {
            let call: CallRequest = parse_param(&req.params, 0)?;
            let tag = parse_block_tag(&req.params, 1)?;
            client.call(&call, tag).await.map(|r| json!(r))
        }
        RpcMethod::EthEstimateGas => {
            let call: CallRequest = parse_param(&req.params, 0)?;
            client.estimate_gas(&call).await.map(|g| json!(format!("0x{:x}", g)))
        }
        RpcMethod::EthGetCode => {
            let addr: Address = parse_param(&req.params, 0)?;
            let tag = parse_block_tag(&req.params, 1)?;
            client.get_code(&addr, tag).await.map(|c| json!(c))
        }
        RpcMethod::EthGetStorageAt => {
            let addr: Address = parse_param(&req.params, 0)?;
            let pos: U256 = parse_param(&req.params, 1)?;
            let tag = parse_block_tag(&req.params, 2)?;
            client.get_storage_at(&addr, &pos, tag).await.map(|s| json!(s))
        }
        RpcMethod::Web3ClientVersion => Ok(json!("Helios/v0.10.2+Tauri")),
        RpcMethod::NetVersion => client.get_chain_id().await.map(|id| json!(id.to_string())),
        RpcMethod::EthSyncing => Ok(json!(false)),

        // ==================== Custom 方法 ====================
        RpcMethod::Custom(m) => match m {
             _ if m.starts_with("trace_") || m.starts_with("debug_") || m == "eth_getLogs" => {
                Err(AppError::UnsupportedMethod(m.to_string()))
            }
            _ => Err(AppError::UnsupportedMethod(m.to_string())),
        },
    };

    match result {
        Ok(value) => JsonRpcResponse {
            jsonrpc: Some("2.0".to_string()),
            id: req.id.map(|v| json!(v)),
            result: Some(value),
            error: None,
        },
        Err(e) => JsonRpcResponse {
            jsonrpc: Some("2.0".to_string()),
            id: req.id.map(|v| json!(v)),
            result: None,
            error: Some(crate::rpc::request::RpcError {
                code: -32603,
                message: e.to_string(),
            }),
        },
    }
}

// ==================== 工具函数 ====================

fn str_to_method(s: &str) -> RpcMethod {
    match s {
        "eth_chainId" => RpcMethod::EthChainId,
        "eth_blockNumber" => RpcMethod::EthBlockNumber,
        "eth_getBalance" => RpcMethod::EthGetBalance,
        "eth_getTransactionCount" => RpcMethod::EthGetTransactionCount,
        "eth_gasPrice" => RpcMethod::EthGasPrice,
        "eth_sendRawTransaction" => RpcMethod::EthSendRawTransaction,
        "eth_getBlockByNumber" => RpcMethod::EthGetBlockByNumber,
        "eth_getBlockByHash" => RpcMethod::EthGetBlockByHash,
        "eth_getTransactionByHash" => RpcMethod::EthGetTransactionByHash,
        "eth_getTransactionReceipt" => RpcMethod::EthGetTransactionReceipt,
        "eth_maxPriorityFeePerGas" => RpcMethod::EthMaxPriorityFeePerGas,
        "eth_feeHistory" => RpcMethod::EthFeeHistory,
        "eth_call" => RpcMethod::EthCall,
        "eth_estimateGas" => RpcMethod::EthEstimateGas,
        "eth_getCode" => RpcMethod::EthGetCode,
        "eth_getStorageAt" => RpcMethod::EthGetStorageAt,
        "web3_clientVersion" => RpcMethod::Web3ClientVersion,
        "net_version" => RpcMethod::NetVersion,
        "eth_syncing" => RpcMethod::EthSyncing,
        _ => RpcMethod::Custom(s),
    }
}

fn parse_param<T: serde::de::DeserializeOwned>(params: &Value, idx: usize) -> Result<T, AppError> {
    params.as_array()
        .and_then(|a| a.get(idx))
        .ok_or(AppError::MissingParam(idx))
        .and_then(|v| serde_json::from_value(v.clone()).map_err(|_| AppError::InvalidParam(idx)))
}

fn parse_block_tag(params: &Value, idx: usize) -> Result<BlockTag, AppError> {
    let s: Option<String> = parse_param(params, idx)?;
    Ok(match s.as_deref() {
        None | Some("latest") => BlockTag::Latest,
        Some("finalized") => BlockTag::Finalized,
        Some("safe") => BlockTag::Safe,
        Some("pending") => BlockTag::Pending,
        Some("earliest") => BlockTag::Earliest,
        Some(h) if h.starts_with("0x") => {
            if h.len() == 66 {
                BlockTag::Hash(h.parse().map_err(|_| AppError::InvalidBlockTag)?)
            } else {
                let n = str_to_u256(h).map_err(|_| AppError::InvalidBlockTag)?;
                BlockTag::Number(n.as_u64())
            }
        }
        _ => return Err(AppError::InvalidBlockTag),
    })
}

/// 将 JsonRpcRequest 转换为 tauri::http::Request<Vec<u8>>
pub fn jsonrpc_request_to_http_request(request: JsonRpcRequest) -> Result<tauri::http::Request<Vec<u8>>, AppError> {
    let body = serde_json::to_vec(&request).map_err(|_| AppError::InvalidParam(0))?;
    let mut http_request = tauri::http::Request::builder()
        .method("POST")
        .header("Content-Type", "application/json")
        .body(body)
        .map_err(|_| AppError::InvalidParam(0))?;
    
    Ok(http_request)
}

/// 将 tauri::http::Response<Vec<u8>> 转换为 JsonRpcResponse
pub fn http_response_to_jsonrpc_response(response: tauri::http::Response<Vec<u8>>) -> Result<JsonRpcResponse, AppError> {
    let body = response.body();
    let json_response: JsonRpcResponse = serde_json::from_slice(body).map_err(|_| AppError::InvalidParam(0))?;
    Ok(json_response)
}

