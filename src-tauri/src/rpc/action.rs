

 // 发送 JSON-RPC 请求（带超时、代理支持和429重试）
    async fn send_rpc_request(&self, rpc_url: &str, method: &str, params: serde_json::Value) -> Result<serde_json::Value> {
        use crate::wallets_tool::ecosystems::ethereum::proxy_manager::PROXY_MANAGER;
        
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: method.to_string(),
            params,
            id: Some(1),
        };

        // 设置10秒超时
        let timeout = Duration::from_secs(10);
        
        // 检查代理配置并获取代理客户端
        let proxy_config = PROXY_MANAGER.get_config();
        let using_proxy = proxy_config.enabled && !proxy_config.proxies.is_empty();
        
        // 优先使用代理客户端，如果没有代理则使用默认客户端
        let client = if let Some(proxy_client) = PROXY_MANAGER.get_random_proxy_client() {
            println!("[DEBUG] 使用代理发送RPC请求 (余额查询): {}", rpc_url);
            if using_proxy {
                println!("[INFO] 代理已启用，当前有 {} 个代理可用", proxy_config.proxies.len());
            }
            proxy_client
        } else {
            if proxy_config.enabled {
                println!("[WARN] 代理已启用但没有可用代理，使用直连模式: {}", rpc_url);
            } else {
                println!("[DEBUG] 代理未启用，使用直连模式发送RPC请求 (余额查询): {}", rpc_url);
            }
            self.client.clone()
        };
        
        // 实现429错误重试机制（最多重试3次）
        let mut retry_count = 0;
        let max_retries = 3;
        
        loop {
            let response = tokio::time::timeout(timeout, 
                client
                    .post(rpc_url)
                    .json(&request)
                    .send()
            ).await
            .map_err(|_| anyhow!("RPC请求超时（10秒），RPC地址: {}", rpc_url))??;

            // 检查是否为429错误
            if response.status().as_u16() == 429 {
                retry_count += 1;
                if retry_count > max_retries {
                    return Err(anyhow!("RPC请求速率限制（429错误），已达到最大重试次数，RPC地址: {}", rpc_url));
                }
                
                // 指数退避：等待时间随重试次数增加而增加
                let wait_time = Duration::from_secs(2_u64.pow(retry_count as u32));
                println!("[WARN] 遇到429速率限制，等待 {:?} 后重试（第 {} 次重试），RPC: {}", wait_time, retry_count, rpc_url);
                sleep(wait_time).await;
                continue;
            }
            
            let json_response: JsonRpcResponse = tokio::time::timeout(timeout,
                response.json::<JsonRpcResponse>()
            ).await
            .map_err(|_| anyhow!("RPC响应解析超时（10秒），RPC地址: {}", rpc_url))??;

            if let Some(error) = json_response.error {
                return Err(anyhow!("RPC Error: {} - {}", error.code, error.message));
            }

            return json_response.result.ok_or_else(|| anyhow!("No result in RPC response"));
        }