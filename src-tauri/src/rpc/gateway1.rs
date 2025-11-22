// src/rpc/gateway.rs

use crate::error::AppError;
use crate::state::{AppState, get_https_client};
use reqwest::{Client, Response};
use serde_json::{json, Value};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tauri::State;
use tokio::sync::RwLock;
use tokio::time::timeout;

#[derive(Debug, Clone)]
pub struct Endpoint {
    pub host: String,        // 完整 URL，如 https://g1.yami.sh/ankr/eth
    pub is_healthy: bool,
    pub latency_ms: u64,     // 最近一次成功延迟
    pub consecutive_fails: u32,
}

#[derive(Clone)]
pub struct GatewayRpc {
    client: Client,
    endpoints: Arc<RwLock<Vec<Endpoint>>>,
    chain_slug: String,      // eth / base / linea 等
}

impl GatewayRpc {
    /// 创建实例 + 立即启动一次健康检查
    pub async fn new(state: &State<'_, AppState>) -> Result<Self, AppError> {
        let session_config = crate::core::state::get_session_config(state)?;
        let chain_id = session_config.current_chain.chain_id;

        let chain_slug = match chain_id {
            1 => "eth",
            56 => "bsc",
            137 => "polygon",
            8453 => "base",
            42161 => "arbitrum",
            59144 => "linea",
            11155111 => "sepolia",
            _ => return Err(AppError::UnsupportedChain(format!("chain_id {}", chain_id))),
        }
        .to_string();

        // 构建完整 endpoints
        let base_hosts = vec![
            "https://g1.yami.sh/ankr",
            "https://g1.yami.sh/drpc",
            "https://g2.yami.sh/ankr",
            "https://g2.yami.sh/drpc",
        ];

        let endpoints: Vec<Endpoint> = base_hosts.into_iter()
            .map(|base| Endpoint {
                host: format!("{}/{}", base, chain_slug),
                is_healthy: false,
                latency_ms: u64::MAX,
                consecutive_fails: 0,
            })
            .collect();

        let rpc = Self {
            client: get_https_client(state)?,
            endpoints: Arc::new(RwLock::new(endpoints)),
            chain_slug: chain_slug.clone(),
        };

        // 启动即测速（不阻塞构造）
        let rpc_clone = rpc.clone();
        tauri::async_runtime::spawn(async move {
            let _ = rpc_clone.refresh_health().await;
        });

        Ok(rpc)
    }

    /// 主动刷新所有节点健康状态（建议每30~60秒后台调用一次）
    pub async fn refresh_health(&self) -> Result<(), AppError> {
        let mut endpoints = self.endpoints.write().await;
        let client = self.client.clone();

        let tasks: Vec<_> = endpoints.iter().enumerate().map(|(idx, ep)| {
            let url = ep.host.clone();
            let client = client.clone();
            async move {
                let start = Instant::now();
                let result = timeout(Duration::from_secs(8), async {
                    let payload = json!({
                        "jsonrpc": "2.0",
                        "method": "eth_blockNumber",
                        "params": [],
                        "id": 1
                    });
                    client.post(&url).json(&payload).send().await?.json::<Value>().await
                }).await;

                let latency = start.elapsed().as_millis() as u64;
                (idx, result.is_ok(), latency)
            }
        }).collect();

        let results = futures::future::join_all(tasks).await;

        for (idx, success, latency) in results {
            if let Some(ep) = endpoints.get_mut(idx) {
                if success {
                    ep.is_healthy = true;
                    ep.latency_ms = latency;
                    ep.consecutive_fails = 0;
                } else {
                    ep.is_healthy = false;
                    ep.consecutive_fails += 1;
                    ep.latency_ms = u64::MAX;
                }
            }
        }

        // 按延迟排序（最快的排前面）
        endpoints.sort_by_key(|ep| {
            if ep.is_healthy {
                ep.latency_ms
            } else {
                u64::MAX
            }
        });

        Ok(())
    }

    /// 选择当前最优节点
    async fn best_endpoint(&self) -> Endpoint {
        let endpoints = self.endpoints.read().await;
        endpoints.iter()
            .filter(|e| e.is_healthy)
            .min_by_key(|e| e.latency_ms)
            .cloned()
            .unwrap_or_else(|| endpoints[0].clone())
    }

    /// 核心请求：优先最快节点 → 失败自动降级
    pub async fn request(&self, method: &str, params: Value) -> Result<Value, AppError> {
        let mut attempts = 0;
        let max_attempts = 4; // 最多尝试 4 个节点

        loop {
            attempts += 1;
            let endpoint = self.best_endpoint().await;

            match self.call_single(&endpoint.host, method, params.clone()).await {
                Ok(result) => {
                    // 成功后立刻提升该节点权重（可选：记录成功延迟用于下次排序）
                    log::debug!("[{}] {} → {} ({}ms)", self.chain_slug, method, endpoint.host, endpoint.latency_ms);
                    return Ok(result);
                }
                Err(e) => {
                    log::warn!("[{}] {} failed on {}: {} (attempt {}/{})", 
                        self.chain_slug, method, endpoint.host, e, attempts, max_attempts);

                    // 标记失败（可选：临时降权）
                    let mut endpoints = self.endpoints.write().await;
                    if let Some(ep) = endpoints.iter_mut().find(|ep| ep.host == endpoint.host) {
                        ep.consecutive_fails += 1;
                        ep.is_healthy = false;
                    }

                    if attempts >= max_attempts {
                        return Err(AppError::AllGatewayEndpointsFailed);
                    }

                    // 小延时避免雪崩
                    tokio::time::sleep(Duration::from_millis(100 * attempts as u64)).await;
                }
            }
        }
    }

    async fn call_single(&self, url: &str, method: &str, params: Value) -> Result<Value, AppError> {
        let payload = json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
            "id": 1
        });

        let resp: Response = timeout(Duration::from_secs(12),
            self.client.post(url)
                .header("Content-Type", "application/json")
                .json(&payload)
                .send()
        ).await
            .map_err(|_| AppError::RpcTimeout)??;

        let json: Value = resp.json().await.map_err(AppError::JsonParseError)?;

        if let Some(err) = json.get("error") {
            let code = err["code"].as_i64().unwrap_or(-1);
            let msg = err["message"].as_str().unwrap_or("unknown").to_string();
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
}

// 让它支持 .clone() 和跨线程
impl Clone for GatewayRpc {
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
            endpoints: self.endpoints.clone(),
            chain_slug: self.chain_slug.clone(),
        }
    }
}

#[tauri::command]
async fn send_tx(state: State<'_, AppState>, signed_tx: String) -> Result<String, AppError> {
    let gateway = GatewayRpc::new(&state).await?;
    
    // 第一次调用前建议手动刷新一次（可选）
    // gateway.refresh_health().await.ok();

    gateway.send_raw_transaction(&signed_tx).await
}