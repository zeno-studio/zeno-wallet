// src/rpc/gateway.rs

use crate::error::AppError;
use crate::state::{AppState, get_gateway_manager, get_https_client};
use reqwest::Client;
use serde_json::{Value, json};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tauri::State;
use tokio::sync::RwLock;

#[derive(Clone, Debug)]
pub struct GatewayNode {
    pub base_url: String, // https://g1.yami.sh/ankr
    pub eth_url: String,  // https://g1.yami.sh/ankr/eth ← 金丝雀测速专用
    pub latency_ms: u64,
    pub healthy: bool,
    pub fails: u32,
}

pub struct GatewayManager {
    pub nodes: RwLock<Vec<GatewayNode>>,
}

impl GatewayManager {
    pub fn new() -> Self {
        let nodes = vec![
            GatewayNode {
                base_url: "https://g1.yami.sh/ankr".into(),
                eth_url: "https://g1.yami.sh/ankr/eth".into(),
                latency_ms: u64::MAX,
                healthy: false,
                fails: 0,
            },
            GatewayNode {
                base_url: "https://g1.yami.sh/drpc".into(),
                eth_url: "https://g1.yami.sh/drpc/eth".into(),
                latency_ms: u64::MAX,
                healthy: false,
                fails: 0,
            },
            GatewayNode {
                base_url: "https://g2.yami.sh/ankr".into(),
                eth_url: "https://g2.yami.sh/ankr/eth".into(),
                latency_ms: u64::MAX,
                healthy: false,
                fails: 0,
            },
            GatewayNode {
                base_url: "https://g2.yami.sh/drpc".into(),
                eth_url: "https://g2.yami.sh/drpc/eth".into(),
                latency_ms: u64::MAX,
                healthy: false,
                fails: 0,
            },
        ];

        let manager = Self {
            nodes: RwLock::new(nodes),
        };

        // 启动全局测速任务（App 启动后只执行一次）
        {
            let mgr = Arc::new(manager);
            tauri::async_runtime::spawn(async move {
                loop {
                    tokio::time::sleep(Duration::from_secs(20)).await;
                    let _ = mgr.refresh_all().await;
                }
            });
        }

        manager
    }

    /// 只测 4 次 eth_blockNumber，代表全部链的延迟
    pub async fn refresh_all(&self) -> Result<(), AppError> {
        let mut nodes = self.nodes.write().await;
        let start = Instant::now();

        let tasks: Vec<_> = nodes
            .iter()
            .enumerate()
            .map(|(i, node)| {
                let url = node.eth_url.clone();
                tokio::spawn(async move {
                    let start = Instant::now();
                    let client = reqwest::Client::new();
                    let ok = client
                        .post(&url)
                        .json(&json!({
                            "jsonrpc": "2.0",
                            "method": "eth_blockNumber",
                            "params": [],
                            "id": 1
                        }))
                        .timeout(Duration::from_secs(10))
                        .send()
                        .await
                        .is_ok_and(|r| r.status().is_success());

                    let latency = start.elapsed().as_millis() as u64;
                    (i, ok, latency)
                })
            })
            .collect();

        let results = futures::future::join_all(tasks).await;

        let mut healthy_count = 0;
        for (idx, ok, latency) in results {
            if let Some(node) = nodes.get_mut(idx) {
                if ok {
                    node.healthy = true;
                    node.latency_ms = latency;
                    node.fails = 0;
                    healthy_count += 1;
                } else {
                    node.healthy = false;
                    node.fails = node.fails.saturating_add(1);
                    node.latency_ms = u64::MAX;
                }
            }
        }

        nodes.sort_by_key(|n| if n.healthy { n.latency_ms } else { u64::MAX });

        Ok(())
    }

    pub async fn get_best_url(&self, chain_slug: &str) -> Option<String> {
        let nodes = self.nodes.read().await;
        nodes
            .iter()
            .find(|n| n.healthy)
            .map(|n| format!("{}/{}", n.base_url, chain_slug))
    }

    pub async fn get_all_healthy_urls(&self, chain_slug: &str) -> Vec<String> {
        let nodes = self.nodes.read().await;
        nodes
            .iter()
            .filter(|n| n.healthy)
            .map(|n| format!("{}/{}", n.base_url, chain_slug))
            .collect()
    }
}

// ========================================
// 下面是通用请求函数，全都走 AppState
// ========================================

/// 核心通用请求：自动选最优节点 + 自动降级
pub async fn request(
    state: State<'_, AppState>,
    chain_slug: &str,
    method: &str,
    params: Value,
) -> Result<Value, AppError> {
    let manager = get_gateway_manager(&state);
    let client = get_https_client(&state);

    let urls = manager.get_all_healthy_urls(chain_slug).await;
    if urls.is_empty() {
        return Err(AppError::NoAvailableGateway(chain_slug.to_string()));
    }

    let mut last_err = None;
    for url in urls {
        match call_single(&client, &url, method, params.clone()).await {
            Ok(result) => return Ok(result),
            Err(e) => {
                log::warn!("[{}] {} failed on {}: {}", chain_slug, method, url, e);
                last_err = Some(e);
            }
        }
    }

    Err(last_err.unwrap_or(AppError::AllGatewayEndpointsFailed))
}

async fn call_single(
    client: &Client,
    url: &str,
    method: &str,
    params: Value,
) -> Result<Value, AppError> {
    let payload = json!({
        "jsonrpc": "2.0",
        "method": method,
        "params": params,
        "id": 1
    });

    let resp = client
        .post(url)
        .json(&payload)
        .timeout(Duration::from_secs(12))
        .send()
        .await
        .map_err(AppError::ReqwestError)?;

    let json: Value = resp.json().await.map_err(AppError::JsonParseError)?;

    if let Some(err) = json.get("error") {
        let code = err["code"].as_i64().unwrap_or(-1);
        let msg = err["message"].as_str().unwrap_or("unknown").to_string();
        return Err(AppError::RpcError(code as u64, msg));
    }

    Ok(json["result"].clone())
}

// ========================================
// 一键业务函数（推荐直接使用这些）
// ========================================

pub async fn block_number(state: State<'_, AppState>, chain: &str) -> Result<u64, AppError> {
    let res = request(state, chain, "eth_blockNumber", json!([])).await?;
    let hex: String = serde_json::from_value(res)?;
    Ok(u64::from_str_radix(&hex.trim_start_matches("0x"), 16)?)
}

pub async fn get_nonce(
    state: State<'_, AppState>,
    chain: &str,
    address: &str,
    block: &str,
) -> Result<u64, AppError> {
    let res = request(
        state,
        chain,
        "eth_getTransactionCount",
        json!([address, block]),
    )
    .await?;
    let hex: String = serde_json::from_value(res)?;
    Ok(u64::from_str_radix(&hex.trim_start_matches("0x"), 16)?)
}

pub async fn send_raw_transaction(
    state: State<'_, AppState>,
    chain: &str,
    signed_tx: &str,
) -> Result<String, AppError> {
    let res = request(state, chain, "eth_sendRawTransaction", json!([signed_tx])).await?;
    Ok(res.as_str().unwrap_or("").to_string())
}

pub async fn get_balance(
    state: State<'_, AppState>,
    chain: &str,
    address: &str,
    block: &str,
) -> Result<String, AppError> {
    let res = request(state, chain, "eth_getBalance", json!([address, block])).await?;
    Ok(res.as_str().unwrap_or("0x0").to_string())
}

#[tauri::command]
async fn send_tx_on_linea(
    state: State<'_, AppState>,
    signed_tx: String,
) -> Result<String, AppError> {
    crate::rpc::gateway::send_raw_transaction(state, "linea", &signed_tx).await
}

#[tauri::command]
async fn get_base_nonce(state: State<'_, AppState>, address: String) -> Result<u64, AppError> {
    crate::rpc::gateway::get_nonce(state, "base", &address, "latest").await
}
