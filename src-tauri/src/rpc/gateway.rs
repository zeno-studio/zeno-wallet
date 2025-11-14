use tauri::command;
use serde::{Serialize, Deserialize};

/// Tauri命令：测试代理连接
#[command]
pub async fn test_proxy_connection(proxy_url: String) -> Result<ProxyTestResult, String> {
    match PROXY_MANAGER.test_proxy(&proxy_url).await {
        Ok((success, latency)) => Ok(ProxyTestResult {
            success,
            latency,
            error: None,
        }),
        Err(error) => Ok(ProxyTestResult {
            success: false,
            latency: 0.0,
            error: Some(error),
        }),
    }
}