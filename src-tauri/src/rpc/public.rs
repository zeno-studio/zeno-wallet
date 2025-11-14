use once_cell::sync::Lazy;
use std::collections::HashMap;

pub static PUBLIC_RPC_ENDPOINTS: Lazy<HashMap<&'static str, Vec<&'static str>>> = Lazy::new(|| {
    HashMap::from([
        ("eth", vec![
            "https://rpc.ankr.com/eth",
            "https://eth-mainnet.public.blastapi.io",
            "https://ethereum.publicnode.com",
        ]),
        ("bsc", vec![
            "https://bsc-dataseed.binance.org",
            "https://rpc.ankr.com/bsc",
            "https://bsc.publicnode.com",
        ]),
        ("polygon", vec![
            "https://polygon-rpc.com",
            "https://rpc.ankr.com/polygon",
            "https://polygon-bor.publicnode.com",
        ]),
    ])
});


#[derive(Debug, Serialize, Deserialize)]
pub struct RpcProviderInfo {
    pub id: i64,
    pub chain_id: i64,
    pub rpc_url: String,
    pub is_active: bool,
    pub priority: i32,
    pub last_success_at: Option<String>,
    pub failure_count: i32,
    pub avg_response_time_ms: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateRpcProviderRequest {
    pub rpc_url: String,
    pub is_active: bool,
    pub priority: i32,
}

#[derive(Debug, Serialize)]
pub struct RpcTestResult {
    pub success: bool,
    pub response_time_ms: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GasPriceInfo {
    pub gas_price_gwei: f64,
    pub chain: String,
}


// 测试 RPC 连接
#[tauri::command]
pub async fn test_rpc_connection(
    rpc_url: String,
) -> Result<RpcTestResult, String> {
    println!("[RPC测试] 开始测试RPC连接: {}", rpc_url);
    let start_time = std::time::Instant::now();
    
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .map_err(|e| {
            let error_msg = format!("创建 HTTP 客户端失败: {}", e);
            println!("[RPC测试] {}", error_msg);
            error_msg
        })?;
    
    let payload = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "eth_blockNumber",
        "params": [],
        "id": 1
    });
    
    println!("[RPC测试] 发送请求到: {}, 请求体: {}", rpc_url, payload);
    
    let success = match client.post(&rpc_url)
        .json(&payload)
        .send()
        .await
    {
        Ok(response) => {
            println!("[RPC测试] 收到响应，状态码: {}", response.status());
            if response.status().is_success() {
                match response.json::<serde_json::Value>().await {
                    Ok(json) => {
                        println!("[RPC测试] 响应JSON: {}", json);
                        let has_result = json.get("result").is_some();
                        println!("[RPC测试] 是否包含result字段: {}", has_result);
                        has_result
                    }
                    Err(e) => {
                        println!("[RPC测试] 解析JSON失败: {}", e);
                        false
                    }
                }
            } else {
                println!("[RPC测试] HTTP状态码不成功: {}", response.status());
                false
            }
        }
        Err(e) => {
            println!("[RPC测试] 请求失败: {}", e);
            false
        }
    };
    
    let response_time_ms = start_time.elapsed().as_millis() as u64;
    
    println!("[RPC测试] 测试完成 - 成功: {}, 响应时间: {}ms", success, response_time_ms);
    
    Ok(RpcTestResult {
        success,
        response_time_ms,
    })
}

pub struct ProviderUtils;

impl ProviderUtils {
    // 获取单个链配置
    pub async fn get_chain_config(chain: &str) -> Result<ChainRpcConfig, String> {
        let configs = get_all_chain_configs()
            .await
            .map_err(|e| e.to_string())?;
        configs
            .get(chain)
            .cloned()
            .ok_or_else(|| format!("不支持的链: {}", chain))
    }

    // 获取链ID
    pub async fn get_chain_id(chain: &str) -> Result<u64, String> {
        let cfg = Self::get_chain_config(chain).await?;
        Ok(cfg.chain_id)
    }
    
    // 随机选择RPC URL
    fn get_random_rpc_url(rpc_urls: &[String]) -> Result<&str, String> {
        if rpc_urls.is_empty() {
            return Err("没有可用的RPC URL".to_string());
        }
        let mut rng = rand::thread_rng();
        Ok(&rpc_urls[rng.gen_range(0..rpc_urls.len())])
    }
    // 获取指定链的Provider
    pub async fn get_provider(chain: &str) -> Result<Provider<Http>, Box<dyn std::error::Error>> {
        use crate::wallets_tool::ecosystems::ethereum::proxy_manager::PROXY_MANAGER;
        
        println!("[DEBUG] get_provider - 开始获取链 '{}' 的Provider", chain);
        
        let chain_config = Self::get_chain_config(chain).await?;
        println!("[DEBUG] get_provider - 获取到链配置，chain_id: {}, rpc_urls数量: {}", 
                 chain_config.chain_id, chain_config.rpc_urls.len());
        
        let rpc_url = Self::get_random_rpc_url(&chain_config.rpc_urls)
            .map_err(|e| format!("获取RPC URL失败: {}. 请检查链 '{}' 是否配置了RPC节点。", e, chain))?;
        println!("[DEBUG] get_provider - 选择的RPC URL: {}", rpc_url);
        
        // 尝试使用代理客户端，如果没有代理则使用默认方式
        let provider = if let Some(proxy_client) = PROXY_MANAGER.get_random_proxy_client() {
            println!("[DEBUG] get_provider - 使用代理客户端创建Provider");
            let url: Url = rpc_url.parse()
                .map_err(|e| format!("Failed to parse RPC URL: {}", e))?;
            let http_provider = Http::new_with_client(url, proxy_client);
            Provider::new(http_provider)
        } else {
            println!("[DEBUG] get_provider - 使用默认方式创建Provider");
            Provider::<Http>::try_from(rpc_url)
                .map_err(|e| {
                    println!("[ERROR] get_provider - Provider创建失败: {}", e);
                    e
                })?
        };
        
        println!("[DEBUG] get_provider - Provider创建成功");
        Ok(provider)
    }
    
    // 获取基础Gas Price
    pub async fn get_base_gas_price(chain: &str) -> Result<f64, Box<dyn std::error::Error>> {
        let provider = Arc::new(Self::get_provider(chain).await?);
        let gas_price = provider.get_gas_price().await?;
        let gas_price_gwei = format_units(gas_price, "gwei")?.parse::<f64>()?;
        Ok(gas_price_gwei)
    }
    
    // 测试RPC连接
    pub async fn test_rpc_connection(rpc_url: &str) -> Result<bool, Box<dyn std::error::Error>> {
        let provider = Provider::<Http>::try_from(rpc_url)?;
        let _block_number = provider.get_block_number().await?;
        Ok(true)
    }
}

// Tauri命令：获取指定链的Gas Price
#[tauri::command]
pub async fn get_chain_gas_price(chain: String) -> Result<GasPriceInfo, String> {
    match ProviderUtils::get_base_gas_price(&chain).await {
        Ok(gas_price) => Ok(GasPriceInfo {
            gas_price_gwei: gas_price,
            chain,
        }),
        Err(e) => Err(e.to_string()),
    }
}

// Tauri命令：测试RPC连接
#[tauri::command]
pub async fn test_rpc_url(rpc_url: String) -> Result<bool, String> {
    match ProviderUtils::test_rpc_connection(&rpc_url).await {
        Ok(result) => Ok(result),
        Err(e) => Err(e.to_string()),
    }
}

// Tauri命令：获取多个链的Gas Price
#[tauri::command]
pub async fn get_multiple_gas_prices(chains: Vec<String>) -> Result<Vec<GasPriceInfo>, String> {
    let mut results = Vec::new();
    
    for chain in chains {
        match ProviderUtils::get_base_gas_price(&chain).await {
            Ok(gas_price) => {
                results.push(GasPriceInfo {
                    gas_price_gwei: gas_price,
                    chain: chain.clone(),
                });
            }
            Err(e) => {
                eprintln!("获取{}链Gas Price失败: {}", chain, e);
                // 继续处理其他链，不中断整个流程
                results.push(GasPriceInfo {
                    gas_price_gwei: 0.0,
                    chain: chain.clone(),
                });
            }
        }
    }
    
    Ok(results)
}