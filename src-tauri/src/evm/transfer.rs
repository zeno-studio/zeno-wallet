

// ERC20 ABI (简化版，只包含必要的函数)
const ERC20_ABI: &str = r#"[
    {
        "constant": true,
        "inputs": [{"name": "_owner", "type": "address"}],
        "name": "balanceOf",
        "outputs": [{"name": "balance", "type": "uint256"}],
        "type": "function"
    },
    {
        "constant": true,
        "inputs": [],
        "name": "decimals",
        "outputs": [{"name": "", "type": "uint8"}],
        "type": "function"
    },
    {
        "constant": true,
        "inputs": [],
        "name": "symbol",
        "outputs": [{"name": "", "type": "string"}],
        "type": "function"
    },
    {
        "constant": false,
        "inputs": [
            {"name": "_to", "type": "address"},
            {"name": "_value", "type": "uint256"}
        ],
        "name": "transfer",
        "outputs": [{"name": "", "type": "bool"}],
        "type": "function"
    }
]"#;

#[derive(Debug, Deserialize)]
struct Erc20Token {
    address: String,
    decimals: u8,
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TokenTransferConfig {
    pub chain: String,
    pub contract_address: String,
    pub delay: [u64; 2],
    pub transfer_type: String, // "1", "2", "3", "4"
    pub transfer_amount: f64,
    pub transfer_amount_list: [f64; 2],
    pub left_amount_list: [f64; 2],
    pub amount_precision: u8,
    pub limit_type: String, // "1", "2", "3"
    pub limit_count: u64,
    pub limit_count_list: [u64; 2],
    pub gas_price_type: String, // "1", "2", "3"
    pub gas_price: f64,
    pub gas_price_rate: f64,
    pub max_gas_price: f64,
    pub error_retry: String,
    pub error_count_limit: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenInfo {
    pub symbol: String,
    pub decimals: u8,
    pub balance: String,
}

// 代币转账工具
pub struct TokenTransferUtils;

impl TokenTransferUtils {
    // 获取代币合约Gas Limit
    pub async fn get_contract_gas_limit(
        config: &TokenTransferConfig,
        provider: Arc<Provider<Http>>,
        _contract_address: Address,
        wallet_address: Address,
        to_address: Address,
        transfer_amount: U256,
    ) -> Result<U256, Box<dyn std::error::Error>> {
        // 将TokenTransferConfig转换为TransferConfig
        let transfer_config = TransferConfig {
            chain: config.chain.clone(),
            delay: config.delay,
            transfer_type: config.transfer_type.clone(),
            transfer_amount: config.transfer_amount,
            transfer_amount_list: config.transfer_amount_list.clone(),
            left_amount_list: config.left_amount_list.clone(),
            amount_precision: config.amount_precision,
            limit_type: config.limit_type.clone(),
            limit_count: config.limit_count,
            limit_count_list: config.limit_count_list.clone(),
            gas_price_type: config.gas_price_type.clone(),
            gas_price: config.gas_price,
            gas_price_rate: config.gas_price_rate,
            max_gas_price: config.max_gas_price,
            error_retry: config.error_retry.clone(),
            error_count_limit: config.error_count_limit,
        };
        
        match config.limit_type.as_str() {
            "1" => {
                // 自动估算Gas Limit
                // 使用新的gas limit函数，传入is_eth=false表示这是代币转账
                let gas_limit = TransferUtils::get_gas_limit_with_token_type(
                    &transfer_config,
                    provider.clone(),
                    wallet_address, // from地址
                    to_address,      // to地址
                    transfer_amount, // 转账金额
                    false // is_eth = false，表示这是代币转账
                ).await?;
                
                Ok(gas_limit)
            }
            "2" => {
                // 使用固定Gas Limit
                Ok(U256::from(config.limit_count))
            }
            "3" => {
                // 使用随机Gas Limit
                let mut rng = rand::thread_rng();
                let gas_limit = rng.gen_range(config.limit_count_list[0]..=config.limit_count_list[1]);
                Ok(U256::from(gas_limit))
            }
            _ => Err("gas limit type error".into()),
        }
    }
}

// Tauri命令：代币转账
#[tauri::command]
pub async fn token_transfer<R: tauri::Runtime>(
    app_handle: tauri::AppHandle<R>,
    index: usize,
    item: TransferItem,
    config: TokenTransferConfig,
) -> Result<TransferResult, String> {
    match token_transfer_internal(app_handle, index, item, config).await {
        Ok(tx_hash) => Ok(TransferResult {
            success: true,
            tx_hash: Some(tx_hash),
            error: None,
        }),
        Err(e) => Ok(TransferResult {
            success: false,
            tx_hash: None,
            error: Some(e.to_string()),
        }),
    }
}

// 内部代币转账实现
async fn token_transfer_internal<R: tauri::Runtime>(
    app_handle: tauri::AppHandle<R>,
    index: usize,
    mut item: TransferItem,
    config: TokenTransferConfig,
) -> Result<String, Box<dyn std::error::Error>> {
    item.retry_flag = false;
    
    // 从数据库获取代币的decimals配置
    let db_manager = get_database_manager();
    let chain_service = ChainService::new(db_manager.get_pool());
    
    let db_decimals = chain_service.get_token_decimals_by_contract(&config.chain, &config.contract_address).await
        .map_err(|e| {
            println!("[ERROR] 从数据库获取decimals失败: {}", e);
            e
        })?;
    
    // 创建Provider
    let provider = create_provider(&config.chain).await.map_err(|e| {
        format!("获取RPC提供商失败: {}", e)
    })?;
    
    // 创建钱包
    if item.private_key.trim().is_empty() {
        return Err("私钥不能为空！".into());
    }
    
    // 处理私钥格式，兼容带0x和不带0x的格式
    let private_key = if item.private_key.starts_with("0x") || item.private_key.starts_with("0X") {
        item.private_key[2..].to_string()
    } else {
        item.private_key.clone()
    };
    
    let wallet = private_key.parse::<LocalWallet>().map_err(|e| {
        format!("私钥格式错误: {}，请检查私钥格式是否正确（应为64位十六进制字符串，可带或不带0x前缀）", e)
    })?;
    let wallet = wallet.with_chain_id(get_rpc_config(&config.chain).await.unwrap().chain_id);
    let wallet_address = wallet.address();
    
    // 解析合约地址和目标地址
    let contract_address: Address = config.contract_address.parse()?;
    let to_address: Address = item.to_addr.parse()?;
    
    // 创建合约实例
    let abi: ethers::abi::Abi = serde_json::from_str(ERC20_ABI)?;
    let contract: Contract<Arc<Provider<Http>>> = Contract::new(contract_address, abi, provider.clone());
    
    // 获取当前使用的RPC URL
    let rpc_url = if let Some(rpc_config) = get_rpc_config(&config.chain).await {
        match rpc_config.get_random_rpc() {
            Ok(url) => url.to_string(),
            Err(e) => format!("获取RPC地址失败: {}", e)
        }
    } else {
        "未知RPC".to_string()
    };
    
    // 获取代币信息
    let balance: U256 = contract.method("balanceOf", wallet_address)?.call().await.map_err(|e| {
        format!("获取代币余额失败 (RPC: {}): {}", rpc_url, e)
    })?;
    
    // 使用数据库配置的decimals值，如果没有则从合约查询
    let decimals = if let Some(db_decimals) = db_decimals {
        println!("[DEBUG] 使用数据库配置的decimals: {}", db_decimals);
        db_decimals as u8
    } else {
        println!("[DEBUG] 数据库中未找到decimals配置，从合约查询...");
        let contract_decimals: u8 = contract.method("decimals", ())?.call().await.map_err(|e| {
            format!("获取代币decimals失败 (RPC: {}): {}", rpc_url, e)
        })?;
        println!("[DEBUG] 合约返回的decimals: {}", contract_decimals);
        contract_decimals
    };
    
    let symbol: String = contract.method("symbol", ())?.call().await.map_err(|e| {
        format!("获取代币symbol失败 (RPC: {}): {}", rpc_url, e)
    })?;
    
    let balance_formatted = format_units(balance, decimals as u32)?;
    
    println!("序号：{}, 当前{}余额为: {}", index, symbol, balance_formatted);
    
    if balance.is_zero() {
        return Err("当前余额不足，不做转账操作！".into());
    }
    
    // 获取Gas Price
    let transfer_config = TransferConfig {
        chain: config.chain.clone(),
        delay: config.delay,
        transfer_type: config.transfer_type.clone(),
        transfer_amount: config.transfer_amount,
        transfer_amount_list: config.transfer_amount_list,
        left_amount_list: config.left_amount_list,
        amount_precision: config.amount_precision,
        limit_type: config.limit_type.clone(),
        limit_count: config.limit_count,
        limit_count_list: config.limit_count_list,
        gas_price_type: config.gas_price_type.clone(),
        gas_price: config.gas_price,
        gas_price_rate: config.gas_price_rate,
        max_gas_price: config.max_gas_price,
        error_retry: config.error_retry.clone(),
        error_count_limit: config.error_count_limit,
    };
    
    // 获取当前使用的RPC URL用于错误信息
    let rpc_url = if let Some(rpc_config) = get_rpc_config(&config.chain).await {
        match rpc_config.get_random_rpc() {
            Ok(url) => url.to_string(),
            Err(e) => format!("获取RPC地址失败: {}", e)
        }
    } else {
        "未知RPC".to_string()
    };
    
    let gas_price = TransferUtils::get_gas_price(&transfer_config, provider.clone()).await.map_err(|e| {
        format!("获取Gas Price失败 (RPC: {}): {}", rpc_url, e)
    })?;
    
    // 计算转账金额
    let transfer_amount = match config.transfer_type.as_str() {
        "1" => {
            // 全部转账
            balance
        }
        "2" => {
            // 转账固定数量
            let amount = parse_units(config.transfer_amount, decimals as u32)?.into();
            if amount >= balance {
                return Err("当前余额不足，不做转账操作！".into());
            }
            amount
        }
        "3" => {
            // 转账随机数量
            let mut rng = rand::thread_rng();
            let random_amount = rng.gen_range(config.transfer_amount_list[0]..=config.transfer_amount_list[1]);
            // 根据精度设置格式化随机金额
            let formatted_amount = format!("{:.precision$}", random_amount, precision = config.amount_precision as usize);
            let precise_amount: f64 = formatted_amount.parse()?;
            let amount = parse_units(precise_amount, decimals as u32)?.into();
            if amount >= balance {
                return Err("当前余额不足，不做转账操作！".into());
            }
            amount
        }
        "4" => {
            // 剩余随机数量
            let balance_f64 = format_units(balance, decimals as u32)?.parse::<f64>()?;
            
            println!("序号：{}, 代币余额: {}", index, balance_f64);
            
            // 检查余额是否足够满足最小剩余数量要求
            if balance_f64 <= config.left_amount_list[1] {
                return Err(format!(
                    "当前代币余额为：{}，无法满足最大剩余数量 {} 要求，不做转账操作！",
                    balance_formatted, config.left_amount_list[1]
                ).into());
            }
            
            let mut rng = rand::thread_rng();
            let left_amount = rng.gen_range(config.left_amount_list[0]..=config.left_amount_list[1]);
            let transfer_amount_f64 = balance_f64 - left_amount;
            
            if transfer_amount_f64 <= 0.0 {
                return Err(format!(
                    "计算转账金额为负数或零：代币余额 {} - 剩余数量 {} = {}，不做转账操作！",
                    balance_f64, left_amount, transfer_amount_f64
                ).into());
            }
            
            // 根据精度设置格式化转账金额
            let formatted_amount = format!("{:.precision$}", transfer_amount_f64, precision = config.amount_precision as usize);
            let precise_amount: f64 = formatted_amount.parse()?;
            
            println!("序号：{}, 剩余数量: {}, 转账金额: {} (格式化后: {})", index, left_amount, transfer_amount_f64, precise_amount);
            
            parse_units(precise_amount, decimals as u32)?.into()
        }
        _ => return Err("无效的转账类型".into()),
    };
    
    let transfer_amount_formatted = format_units(transfer_amount, decimals as u32)?;
    println!("序号：{}, 转账数量为: {} {}", index, transfer_amount_formatted, symbol);
    
    // 获取Gas Limit
    let gas_limit = TokenTransferUtils::get_contract_gas_limit(
        &config,
        provider.clone(),
        contract_address,
        wallet_address,
        to_address,
        transfer_amount,
    ).await?;
    
    println!("序号：{}, gasLimit: {}", index, gas_limit);
    
    // 构建转账交易前的详细验证和日志
    println!("[DEBUG] ===== 转账交易构建阶段 =====");
    println!("[DEBUG] 序号: {}", index);
    println!("[DEBUG] 合约地址: {:?}", contract_address);
    println!("[DEBUG] 发送方地址: {:?}", wallet_address);
    println!("[DEBUG] 接收方地址: {:?}", to_address);
    println!("[DEBUG] 转账金额: {} (原始值: {})", transfer_amount_formatted, transfer_amount);
    println!("[DEBUG] Gas Price: {} wei", gas_price);
    println!("[DEBUG] Gas Limit: {}", gas_limit);
    
    // 验证地址有效性
    if wallet_address == Address::zero() {
        return Err("发送方地址无效（零地址）".into());
    }
    if to_address == Address::zero() {
        return Err("接收方地址无效（零地址）".into());
    }
    if contract_address == Address::zero() {
        return Err("合约地址无效（零地址）".into());
    }
    
    // 验证转账金额
    if transfer_amount.is_zero() {
        return Err("转账金额不能为零".into());
    }
    
    // 获取发送方平台币余额用于gas费检查
    let wallet_balance = provider.get_balance(wallet_address, None).await.map_err(|e| {
        format!("获取平台币余额失败 (RPC: {}): {}", rpc_url, e)
    })?;
    let estimated_gas_fee = gas_price * gas_limit;
    
    // 格式化为可读的单位
    let balance_formatted = format_units(wallet_balance, 18).unwrap_or_else(|_| "N/A".to_string());
    let gas_fee_formatted = format_units(estimated_gas_fee, 18).unwrap_or_else(|_| "N/A".to_string());
    let gas_price_gwei = format_units(gas_price, "gwei").unwrap_or_else(|_| "N/A".to_string());
    
    println!("[DEBUG] 平台币余额: {} wei ({} BNB/ETH)", wallet_balance, balance_formatted);
    println!("[DEBUG] Gas Price: {} wei ({} gwei)", gas_price, gas_price_gwei);
    println!("[DEBUG] Gas Limit: {}", gas_limit);
    println!("[DEBUG] 预估Gas费用: {} wei ({} BNB/ETH)", estimated_gas_fee, gas_fee_formatted);
    
    // 获取链ID以进行特殊处理
    let chain_id = provider.get_chainid().await.unwrap_or_default().as_u64();
    
    // 对BSC链进行特殊处理，使用更宽松的余额检查
    if chain_id == 56 || chain_id == 97 {
        // BSC链的余额检查应该更加宽松，考虑到可能的计算误差
        let buffer_percentage = U256::from(110); // 10%的缓冲
        let buffered_gas_fee = estimated_gas_fee * buffer_percentage / U256::from(100);
        
        println!("[DEBUG] BSC链特殊处理 - 原始预估Gas费用: {}, 带缓冲的Gas费用: {}", estimated_gas_fee, buffered_gas_fee);
        
        if wallet_balance < buffered_gas_fee {
            return Err(format!(
                "平台币余额不足支付Gas费用！\n当前余额: {} ({} wei)\n预估Gas费用: {} ({} wei)\nGas Price: {} gwei, Gas Limit: {}\n(已考虑10%缓冲)",
                balance_formatted, wallet_balance,
                gas_fee_formatted, estimated_gas_fee,
                gas_price_gwei, gas_limit
            ).into());
        }
    } else {
        // 其他链使用标准检查
        if wallet_balance < estimated_gas_fee {
            return Err(format!(
                "平台币余额不足支付Gas费用！\n当前余额: {} ({} wei)\n预估Gas费用: {} ({} wei)\nGas Price: {} gwei, Gas Limit: {}",
                balance_formatted, wallet_balance,
                gas_fee_formatted, estimated_gas_fee,
                gas_price_gwei, gas_limit
            ).into());
        }
    }
    
    // 构建转账交易
    let client = SignerMiddleware::new(provider.clone(), wallet);
    let contract_with_signer: Contract<Arc<SignerMiddleware<Arc<Provider<Http>>, LocalWallet>>> = Contract::new(contract_address, contract.abi().clone(), Arc::new(client));
    
    item.error_msg = "发送交易...".to_string();
    // 发送状态更新事件到前端
    let _ = app_handle.emit("transfer_status_update", serde_json::json!({
        "index": index - 1,
        "error_msg": item.error_msg.clone(),
        "exec_status": "1"
    }));
    
    println!("[DEBUG] ===== 发送交易阶段 =====");
    
    // 调用transfer方法
    let call = contract_with_signer
        .method::<_, bool>("transfer", (to_address, transfer_amount))?
        .gas_price(gas_price)
        .gas(gas_limit);
    
    println!("[DEBUG] 交易调用已构建，准备发送...");
    
    let pending_tx = match call.send().await {
        Ok(tx) => {
            println!("[DEBUG] 交易发送成功，等待确认...");
            tx
        }
        Err(e) => {
            // 获取当前使用的RPC URL
            let rpc_url = if let Some(rpc_config) = get_rpc_config(&config.chain).await {
                match rpc_config.get_random_rpc() {
                    Ok(url) => url.to_string(),
                    Err(e) => format!("获取RPC地址失败: {}", e)
                }
            } else {
                "未知RPC".to_string()
            };
            
            let error_msg = format!("发送交易失败 (RPC: {}): {}", rpc_url, e);
            println!("[ERROR] {}", error_msg);
            
            // 分析具体的错误类型
            let detailed_error = if e.to_string().contains("insufficient funds") {
                format!("余额不足 (RPC: {}): {}", rpc_url, e)
            } else if e.to_string().contains("gas") {
                format!("Gas相关错误 (RPC: {}): {}", rpc_url, e)
            } else if e.to_string().contains("revert") {
                format!("合约执行被回滚 (RPC: {}): {}", rpc_url, e)
            } else if e.to_string().contains("nonce") {
                format!("Nonce错误 (RPC: {}): {}", rpc_url, e)
            } else {
                format!("网络或其他错误 (RPC: {}): {}", rpc_url, e)
            };
            
            return Err(detailed_error.into());
        }
    };
    
    let tx_hash = pending_tx.tx_hash();
    println!("序号：{}, 交易 hash 为：{:?}", index, tx_hash);
    
    // 等待交易确认（设置30秒超时）
    item.error_msg = "等待交易结果...".to_string();
    // 发送状态更新事件到前端
    let _ = app_handle.emit("transfer_status_update", serde_json::json!({
        "index": index - 1,
        "error_msg": item.error_msg.clone(),
        "exec_status": "1"
    }));
    
    println!("[DEBUG] ===== 等待交易确认阶段 =====");
    println!("[DEBUG] 交易哈希: {:?}", tx_hash);
    println!("[DEBUG] 开始等待交易确认，设置30秒超时...");
    
    // 获取RPC URL用于错误消息
    let rpc_url_for_error = if let Some(rpc_config) = get_rpc_config(&config.chain).await {
        match rpc_config.get_random_rpc() {
            Ok(url) => url.to_string(),
            Err(e) => format!("获取RPC地址失败: {}", e)
        }
    } else {
        "未知RPC".to_string()
    };
    
    let receipt = match tokio::time::timeout(
        tokio::time::Duration::from_secs(30),
        pending_tx
    ).await {
        Ok(result) => {
            result.map_err(|e| {
                let error_msg = format!("等待交易确认失败 (RPC: {}) (交易哈希: {:?}): {}", rpc_url_for_error, tx_hash, e);
                println!("[ERROR] {}", error_msg);
                error_msg
            })?
        }
        Err(_) => {
            // 超时处理
            let timeout_msg = format!("等待交易确认超时 (RPC: {}) - 超过30秒未收到确认，交易哈希: {:?}", rpc_url_for_error, tx_hash);
            println!("[ERROR] {}", timeout_msg);
            return Err(timeout_msg.into());
        }
    };
    
    match receipt {
        Some(receipt) => {
            println!("[DEBUG] ===== 交易收据分析 =====");
            println!("[DEBUG] 交易哈希: {:?}", receipt.transaction_hash);
            println!("[DEBUG] 区块号: {:?}", receipt.block_number);
            println!("[DEBUG] Gas使用量: {:?}", receipt.gas_used);
            println!("[DEBUG] 交易状态: {:?}", receipt.status);
            println!("[DEBUG] 累积Gas使用量: {:?}", receipt.cumulative_gas_used);
            
            if receipt.status == Some(U64::from(1)) {
                println!("[INFO] 交易执行成功！");
                Ok(format!("{:?}", receipt.transaction_hash))
            } else {
                // 获取当前使用的RPC URL
                let rpc_url = if let Some(rpc_config) = get_rpc_config(&config.chain).await {
                    match rpc_config.get_random_rpc() {
                        Ok(url) => url.to_string(),
                        Err(e) => format!("获取RPC地址失败: {}", e)
                    }
                } else {
                    "未知RPC".to_string()
                };
                
                let error_msg = format!(
                    "交易执行失败 (RPC: {}) - 交易哈希: {:?}, 区块号: {:?}, Gas使用: {:?}/{}, 状态: {:?}",
                    rpc_url,
                    receipt.transaction_hash,
                    receipt.block_number.unwrap_or_default(),
                    receipt.gas_used.unwrap_or_default(),
                    gas_limit,
                    receipt.status.unwrap_or_default()
                );
                println!("[ERROR] {}", error_msg);
                
                // 尝试获取更详细的失败原因
                let detailed_error = if let Some(gas_used) = receipt.gas_used {
                    if gas_used >= gas_limit {
                        format!("{} (可能原因: Gas不足，已用完所有Gas)", error_msg)
                    } else {
                        format!("{} (可能原因: 合约执行被回滚)", error_msg)
                    }
                } else {
                    error_msg
                };
                
                Err(detailed_error.into())
            }
        }
        None => {
            // 获取当前使用的RPC URL
            let rpc_url = if let Some(rpc_config) = get_rpc_config(&config.chain).await {
                match rpc_config.get_random_rpc() {
                    Ok(url) => url.to_string(),
                    Err(e) => format!("获取RPC地址失败: {}", e)
                }
            } else {
                "未知RPC".to_string()
            };
            let error_msg = format!("交易未确认 (RPC: {}) (交易哈希: {:?}) - 可能网络拥堵或交易被丢弃", rpc_url, tx_hash);
            println!("[ERROR] {}", error_msg);
            Err(error_msg.into())
        }
    }
}

// Tauri命令：获取代币信息
#[tauri::command]
pub async fn get_token_info(
    chain: String,
    contract_address: String,
) -> Result<TokenInfo, String> {
    match get_token_info_internal(chain, contract_address).await {
        Ok(token_info) => Ok(token_info),
        Err(e) => Err(e.to_string()),
    }
}

// 内部获取代币信息实现
async fn get_token_info_internal(
    chain: String,
    contract_address: String,
) -> Result<TokenInfo, Box<dyn std::error::Error>> {
    let provider = create_provider(&chain).await?;
    
    let contract_addr: Address = contract_address.parse()?;
    
    // 创建合约实例
    let abi: ethers::abi::Abi = serde_json::from_str(ERC20_ABI)?;
    let contract: Contract<Arc<Provider<Http>>> = Contract::new(contract_addr, abi, provider);
    
    // 获取代币信息
    let decimals: u8 = contract.method("decimals", ())?.call().await?;
    let symbol: String = contract.method("symbol", ())?.call().await?;
    
    Ok(TokenInfo {
        symbol,
        decimals,
        balance: "0".to_string(), // 不查询余额，只获取基本信息
    })
}


// 转账工具函数
pub struct TransferUtils;

impl TransferUtils {
    // 获取当前网络的baseFee
    pub async fn get_base_fee(
        provider: Arc<Provider<Http>>,
    ) -> Result<U256, Box<dyn std::error::Error>> {
        // 获取最新区块
        let latest_block = provider.get_block(BlockNumber::Latest).await?;
        
        if let Some(block) = latest_block {
            if let Some(base_fee) = block.base_fee_per_gas {
                println!("[DEBUG] 获取到当前baseFee: {} wei ({} gwei)", 
                    base_fee, 
                    format_units(base_fee, "gwei").unwrap_or_default()
                );
                return Ok(base_fee);
            }
        }
        
        // 如果无法获取baseFee，返回默认值（适用于非EIP-1559网络）
        println!("[DEBUG] 无法获取baseFee，使用默认值0");
        Ok(U256::zero())
    }

    // 获取区块Gas Limit
    pub async fn get_block_gas_limit(
        provider: Arc<Provider<Http>>,
    ) -> Result<U256, Box<dyn std::error::Error>> {
        match provider.get_block(BlockNumber::Latest).await {
            Ok(Some(block)) => {
                let raw_gas_limit = block.gas_limit;
                println!("[DEBUG] 从RPC获取到的原始区块gas limit: {}", raw_gas_limit);
                
                // 合理性检查：如果gas limit超过1亿，认为是异常值
                let max_reasonable_gas_limit = U256::from(100_000_000u64); // 1亿
                
                if raw_gas_limit > max_reasonable_gas_limit {
                    println!("[WARN] 检测到异常的区块gas limit: {}，远超合理范围", raw_gas_limit);
                    
                    // 根据链ID返回合理的默认值
                    let chain_id = match provider.get_chainid().await {
                        Ok(id) => id.as_u64(),
                        Err(_) => 0,
                    };
                    
                    let default_gas_limit = match chain_id {
                        42161 => U256::from(30_000_000u64), // Arbitrum One
                        1 => U256::from(30_000_000u64),     // Ethereum Mainnet
                        137 => U256::from(30_000_000u64),   // Polygon
                        56 => U256::from(140_000_000u64),   // BSC (更高的gas limit)
                        _ => U256::from(30_000_000u64),     // 其他链的默认值
                    };
                    
                    println!("[INFO] 使用链ID {} 的默认gas limit: {}", chain_id, default_gas_limit);
                    Ok(default_gas_limit)
                } else {
                    println!("[DEBUG] 区块gas limit正常: {}", raw_gas_limit);
                    Ok(raw_gas_limit)
                }
            }
            Ok(None) => {
                eprintln!("[ERROR] 无法获取最新区块信息");
                Err("无法获取最新区块信息".into())
            }
            Err(e) => {
                eprintln!("[ERROR] 获取区块gas limit失败: {}", e);
                Err(format!("获取区块gas limit失败: {}", e).into())
            }
        }
    }

    // 获取最近三个区块中所有transfer交易的平均gas limit
    pub async fn get_average_gas_limit_from_recent_blocks(
        provider: Arc<Provider<Http>>,
    ) -> Result<U256, Box<dyn std::error::Error>> {
        let mut total_gas_used = U256::zero();
        let mut transaction_count = 0u64;
        
        // 获取最新区块号
        let latest_block_number = match provider.get_block_number().await {
            Ok(block_num) => block_num,
            Err(e) => {
                eprintln!("获取最新区块号失败: {}", e);
                return Err(format!("获取最新区块号失败: {}", e).into());
            }
        };
        
        println!("开始分析最近3个区块的transfer交易，当前区块号: {}", latest_block_number);
        
        // 遍历最近3个区块
        for i in 0..3 {
            if latest_block_number < U64::from(i) {
                break; // 避免区块号下溢
            }
            
            let block_number = latest_block_number - U64::from(i);
            
            match provider.get_block_with_txs(BlockNumber::Number(block_number)).await {
                Ok(Some(block)) => {
                    println!("分析区块 {} 中的 {} 个交易", block_number, block.transactions.len());
                    
                    // 遍历区块中的所有交易
                    for tx in &block.transactions {
                        // 检查是否为transfer交易（有to地址且value > 0或者是代币转账）
                        let is_transfer = tx.to.is_some() && 
                            (tx.value > U256::zero() || 
                             (tx.input.len() >= 4 && 
                              (&tx.input[0..4] == [0xa9, 0x05, 0x9c, 0xbb] || // transfer(address,uint256)
                               &tx.input[0..4] == [0x23, 0xb8, 0x72, 0xdd])))  // transferFrom(address,address,uint256)
                        ;
                        
                        if is_transfer {
                            total_gas_used += tx.gas;
                            transaction_count += 1;
                        }
                    }
                }
                Ok(None) => {
                    eprintln!("区块 {} 不存在", block_number);
                }
                Err(e) => {
                    eprintln!("获取区块 {} 失败: {}", block_number, e);
                }
            }
        }
        
        if transaction_count == 0 {
            println!("最近3个区块中未找到transfer交易，使用默认值");
            return Err("最近3个区块中未找到transfer交易".into());
        }
        
        let average_gas_limit = total_gas_used / U256::from(transaction_count);
        println!("分析了 {} 个transfer交易，平均gas limit: {}", transaction_count, average_gas_limit);
        
        Ok(average_gas_limit)
    }
    // 预检查余额是否充足（在实际转账前进行检查，避免RPC调用后才发现余额不足）
    pub async fn pre_check_balance(
        config: &TransferConfig,
        provider: Arc<Provider<Http>>,
        wallet_address: Address,
        to_address: Address,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // 获取当前余额
        let balance = match provider.get_balance(wallet_address, None).await {
            Ok(balance) => balance,
            Err(e) => {
                // 获取当前使用的RPC URL
                let error_msg = e.to_string();
                let rpc_url = if let Some(rpc_config) = get_rpc_config(&config.chain).await {
                    match rpc_config.get_random_rpc() {
                        Ok(url) => url.to_string(),
                        Err(e) => format!("获取RPC地址失败: {}", e)
                    }
                } else {
                    "未知RPC".to_string()
                };
                return Err(format!("获取钱包余额失败 (RPC: {}): {}", rpc_url, error_msg).into());
            }
        };
        
        if balance.is_zero() {
            return Err("当前余额为0，无法进行转账操作！".into());
        }
        
        // 获取Gas Price进行预估
        let gas_price = Self::get_gas_price(config, provider.clone()).await?;
        
        if gas_price.is_zero() {
            return Err("获取到的 gas price 为0，请检查网络连接或RPC配置".into());
        }
        // 根据转账类型进行不同的余额检查
        match config.transfer_type.as_str() {
            "1" => {
                // 全部转账 - 需要预留Gas费用
                let estimated_gas_limit = Self::get_gas_limit(config, provider.clone(), wallet_address, to_address, parse_ether(0.000003)?).await?;
          
                print!("estimated_gas_limit: {}", estimated_gas_limit);
                let estimated_gas_fee = gas_price * estimated_gas_limit;
                if estimated_gas_fee >= balance {
                    return Err(format!(
                        "余额不足支付Gas费用！当前余额: {} ETH，预估Gas费用: {} ETH",
                        format_units(balance, 18)?,
                        format_units(estimated_gas_fee, 18)?
                    ).into());
                }
            }
            "2" => {
                // 转账固定数量
                let transfer_amount = parse_ether(config.transfer_amount)?;
                let estimated_gas_limit = Self::get_gas_limit(config, provider.clone(), wallet_address, to_address, parse_ether(0.000003)?).await?;
                let estimated_gas_fee = gas_price * estimated_gas_limit;
                let total_needed = transfer_amount + estimated_gas_fee;
                print!("estimated_gas_limit: {}", estimated_gas_limit);
                if total_needed > balance {
                    return Err(format!(
                        "余额不足！需要: {} ETH (转账: {} + Gas: {} ETH)，当前余额: {} ETH",
                        format_units(total_needed, 18)?,
                        format_units(transfer_amount, 18)?,
                        format_units(estimated_gas_fee, 18)?,
                        format_units(balance, 18)?
                    ).into());
                }
            }
            "3" => {
                // 转账随机数量 - 使用最大可能金额进行检查
                let max_transfer_amount = parse_ether(config.transfer_amount_list[1])?;
                let estimated_gas_limit =Self::get_gas_limit(config, provider.clone(), wallet_address, to_address, parse_ether(0.000003)?).await?;
                let estimated_gas_fee = gas_price * estimated_gas_limit;
                let total_needed = max_transfer_amount + estimated_gas_fee;
                
                if total_needed > balance {
                    return Err(format!(
                        "余额不足！最大可能需要: {} ETH (转账: {} + Gas: {} ETH)，当前余额: {} ETH",
                        format_units(total_needed, 18)?,
                        format_units(max_transfer_amount, 18)?,
                        format_units(estimated_gas_fee, 18)?,
                        format_units(balance, 18)?
                    ).into());
                }
            }
            "4" => {
                // 剩余随机数量 - 检查是否有足够余额满足最小剩余要求
                 let estimated_gas_limit = match config.limit_type.as_str() {
                    "1" => {
                        // 自动估算模式，使用最小转账金额进行估算
                        let min_transfer = parse_ether(0.000003)?;
                        Self::get_gas_limit(config, provider.clone(), wallet_address, to_address, min_transfer).await?
                    }
                    "2" => U256::from(config.limit_count),
                    "3" => {
                        // 使用最大可能的gas limit进行保守估算
                        U256::from(config.limit_count_list[1])
                    }
                    _ => U256::from(21000), // 默认ETH转账gas limit
                };
                let estimated_gas_fee = gas_price * estimated_gas_limit;
                let balance_ether = format_units(balance, 18)?.parse::<f64>()?;
                let gas_fee_ether = format_units(estimated_gas_fee, 18)?.parse::<f64>()?;
                let available_balance = balance_ether - gas_fee_ether;
                
                if available_balance <= config.left_amount_list[1] {
                    return Err(format!(
                        "余额不足！可用余额: {} ETH (总余额: {} - Gas: {} ETH)，无法满足最大剩余数量 {} ETH 要求",
                        available_balance, balance_ether, gas_fee_ether, config.left_amount_list[1]
                    ).into());
                }
            }
            _ => return Err("无效的转账类型".into()),
        }
        
        Ok(())
    }

    // 获取Gas Price
    pub async fn get_gas_price(
        config: &TransferConfig,
        provider: Arc<Provider<Http>>,
    ) -> Result<U256, Box<dyn std::error::Error>> {
        // 获取当前网络的baseFee
        let base_fee = if let Ok(fee) = Self::get_base_fee(provider.clone()).await {
            fee
        } else {
            // 获取当前使用的RPC URL
            let rpc_url = if let Some(rpc_config) = get_rpc_config(&config.chain).await {
                match rpc_config.get_random_rpc() {
                    Ok(url) => url.to_string(),
                    Err(e) => format!("获取RPC地址失败: {}", e)
                }
            } else {
                "未知RPC".to_string()
            };
            println!("[WARN] 获取Base Fee失败 (RPC: {}), 使用默认值0", rpc_url);
            U256::zero()
        };
        
        // 获取链ID并判断链类型
        let chain_id = provider.get_chainid().await?;
        let chain_id_u64 = chain_id.as_u64();
        let is_arbitrum = chain_id_u64 == 42161; // Arbitrum One
        
        // 判断是否为真正的EIP-1559链
        // BSC(56)虽然可能返回baseFee，但实际上不是标准的EIP-1559链，其Gas Price机制不同
        let is_eip1559_chain = match chain_id_u64 {
            1 => true,      // Ethereum Mainnet
            5 => true,      // Goerli
            11155111 => true, // Sepolia
            137 => true,    // Polygon
            42161 => true,  // Arbitrum One
            10 => true,     // Optimism
            56 => false,    // BSC - 非EIP-1559链
            97 => false,    // BSC Testnet - 非EIP-1559链
            _ => base_fee > U256::zero(), // 其他链根据是否有baseFee判断
        };
        
        println!("[DEBUG] 链ID: {}, 是否为Arbitrum: {}, 是否为EIP-1559链: {}", chain_id_u64, is_arbitrum, is_eip1559_chain);
        
        let calculated_gas_price = match config.gas_price_type.as_str() {
            "1" => {
                // 使用网络Gas Price
                let gas_price = match provider.get_gas_price().await {
                    Ok(price) => price,
                    Err(e) => {
                        // 获取当前使用的RPC URL
                        let error_msg = e.to_string();
                        let rpc_url = if let Some(rpc_config) = get_rpc_config(&config.chain).await {
                            match rpc_config.get_random_rpc() {
                                Ok(url) => url.to_string(),
                                Err(e) => format!("获取RPC地址失败: {}", e)
                            }
                        } else {
                            "未知RPC".to_string()
                        };
                        return Err(format!("获取网络Gas Price失败 (RPC: {}): {}", rpc_url, error_msg).into());
                    }
                };
                
                // 检查最大Gas Price限制
                if config.max_gas_price > 0.0 {
                    let gas_price_gwei = format_units(gas_price, "gwei")?.parse::<f64>()?;
                    if gas_price_gwei > config.max_gas_price {
                        return Err("base gas price 超出最大值限制".into());
                    }
                }
                
                gas_price
            }
            "2" => {
                // 使用固定Gas Price
                parse_units(config.gas_price, "gwei")?.into()
            }
            "3" => {
                // 使用溢价Gas Price
                let base_gas_price = match provider.get_gas_price().await {
                    Ok(price) => price,
                    Err(e) => {
                        // 获取当前使用的RPC URL
                        let error_msg = e.to_string();
                        let rpc_url = if let Some(rpc_config) = get_rpc_config(&config.chain).await {
                            match rpc_config.get_random_rpc() {
                                Ok(url) => url.to_string(),
                                Err(e) => format!("获取RPC地址失败: {}", e)
                            }
                        } else {
                            "未知RPC".to_string()
                        };
                        return Err(format!("获取基础Gas Price失败 (RPC: {}): {}", rpc_url, error_msg).into());
                    }
                };
                
                // 安全地计算gas price rate，避免溢出
                let rate_percentage = config.gas_price_rate * 100.0;
                if rate_percentage < 0.0 || rate_percentage > f64::MAX / 2.0 {
                    println!("[ERROR] Gas price rate 值异常: {}", rate_percentage);
                    return Err(format!("Gas price rate 值异常: {}", rate_percentage).into());
                }
                
                // 使用U256进行安全计算，避免u64溢出
                let rate_u256 = U256::from((rate_percentage as u64).min(u64::MAX));
                let multiplier = U256::from(100) + rate_u256;
                let gas_price_with_rate = base_gas_price * multiplier / U256::from(100);
                
                // 检查最大Gas Price限制
                if config.max_gas_price > 0.0 {
                    let base_gas_price_gwei = format_units(base_gas_price, "gwei")?.parse::<f64>()?;
                    if base_gas_price_gwei > config.max_gas_price {
                        return Err("base gas price 超出最大值限制".into());
                    }
                    
                    let final_gas_price_gwei = format_units(gas_price_with_rate, "gwei")?.parse::<f64>()?;
                    if final_gas_price_gwei >= config.max_gas_price {
                        return Ok(parse_units(config.max_gas_price, "gwei")?.into());
                    }
                }
                
                gas_price_with_rate
            }
            _ => return Err("gas price type error".into()),
        };
        
        // 对于BSC链，使用更合理的Gas Price计算方式
        if chain_id_u64 == 56 || chain_id_u64 == 97 {
            // BSC链的特殊处理：直接返回计算的Gas Price，不使用baseFee
            println!("[DEBUG] BSC链特殊处理，直接使用计算的Gas Price: {} gwei", 
                format_units(calculated_gas_price, "gwei").unwrap_or_default()
            );
            return Ok(calculated_gas_price);
        }
        
        // 确保Gas Price高于baseFee（仅对真正的EIP-1559链生效）
        if is_eip1559_chain && base_fee > U256::zero() {
            let min_gas_price = if is_arbitrum {
                // Arbitrum链：baseFee * 1.5 (50%安全边际)
                base_fee * U256::from(150) / U256::from(100)
            } else {
                // 其他EIP-1559链：baseFee * 1.2 (20%安全边际)
                base_fee * U256::from(120) / U256::from(100)
            };
            
            let final_gas_price = if calculated_gas_price < min_gas_price {
                println!("[DEBUG] 计算的Gas Price ({} gwei) 低于最小要求 ({} gwei)，使用最小值", 
                    format_units(calculated_gas_price, "gwei").unwrap_or_default(),
                    format_units(min_gas_price, "gwei").unwrap_or_default()
                );
                min_gas_price
            } else {
                calculated_gas_price
            };
            
            println!("[DEBUG] 最终Gas Price: {} gwei (baseFee: {} gwei)", 
                format_units(final_gas_price, "gwei").unwrap_or_default(),
                format_units(base_fee, "gwei").unwrap_or_default()
            );
            
            Ok(final_gas_price)
        } else {
            // 非EIP-1559网络（如BSC），直接使用计算的Gas Price，不受baseFee影响
            println!("[DEBUG] 非EIP-1559网络或无baseFee，使用计算的Gas Price: {} gwei", 
                format_units(calculated_gas_price, "gwei").unwrap_or_default()
            );
            Ok(calculated_gas_price)
        }
    }

    // 获取Gas Limit
    pub async fn get_gas_limit(
        config: &TransferConfig,
        provider: Arc<Provider<Http>>,
        from: Address,
        to: Address,
        value: U256,
    ) -> Result<U256, String> {
        // 根据链配置的原生货币符号判断是否为ETH转账
        let chain_config = ProviderUtils::get_chain_config(&config.chain).await
            .map_err(|e| format!("获取链配置失败: {}", e))?;
        let is_eth = chain_config.currency_symbol == "ETH";
        Self::get_gas_limit_with_token_type(config, provider, from, to, value, is_eth).await
    }

    // 获取Gas Limit（支持区分代币类型）
    pub async fn get_gas_limit_with_token_type(
        config: &TransferConfig,
        provider: Arc<Provider<Http>>,
        from: Address,
        to: Address,
        value: U256,
        is_eth: bool,
    ) -> Result<U256, String> {
        // 首先获取区块gas limit作为上限检查
        let block_gas_limit = match Self::get_block_gas_limit(provider.clone()).await {
            Ok(limit) => {
                println!("[DEBUG] 获取到区块gas limit: {}", limit);
                limit
            }
            Err(e) => {
                println!("[WARN] 获取区块gas limit失败: {}, 使用默认上限", e);
                U256::from(30_000_000) // 使用一个合理的默认上限
            }
        };
        
        // 计算区块gas limit的80%作为安全上限
        let max_safe_gas_limit = block_gas_limit * U256::from(80) / U256::from(100);
        println!("[DEBUG] 区块gas limit安全上限(80%): {}", max_safe_gas_limit);
        
        match config.limit_type.as_str() {
            "1" => {
                // 自动估算Gas Limit
                let tx = TransactionRequest::new()
                    .from(from)
                    .to(to)
                    .value(value);
                
                let estimated_gas = match provider.estimate_gas(&tx.into(), None).await {
                    Ok(gas) => {
                        println!("[DEBUG] estimate_gas成功: {}", gas);
                        gas
                    }
                    Err(e) => {
                        // 获取当前使用的RPC URL
                        let error_msg = e.to_string();
                        let rpc_url = if let Some(rpc_config) = get_rpc_config(&config.chain).await {
                            match rpc_config.get_random_rpc() {
                                Ok(url) => url.to_string(),
                                Err(e) => format!("获取RPC地址失败: {}", e)
                            }
                        } else {
                            "未知RPC".to_string()
                        };
                        println!("[WARN] estimate_gas失败 (RPC: {}): {}, 使用默认值", rpc_url, error_msg);
                        // 直接使用默认值，不再尝试获取平均gas limit（因为tx.gas是gas limit而非实际使用值）
                        {
                            println!("[WARN] 获取平均gas limit也失败，使用固定默认值");
                            // 直接使用合理的固定默认值，不使用区块gas limit计算
                            if is_eth {
                                // ETH转账
                                U256::from(21_000)
                            } else {
                                // 代币转账根据链类型设置默认值
                                let chain_id = provider.get_chainid().await.unwrap_or_default().as_u64();
                                let default_token_gas = match chain_id {
                                    42161 => U256::from(150_000),  // Arbitrum One - 更高的默认值
                                    1 => U256::from(65_000),       // Ethereum Mainnet
                                    137 => U256::from(65_000),     // Polygon
                                    56 => U256::from(60_000),      // BSC
                                    _ => U256::from(80_000),       // 其他链使用保守值
                                };
                                println!("[DEBUG] 链ID: {}, 代币转账默认Gas Limit: {}", chain_id, default_token_gas);
                                default_token_gas
                            }
                        }
                    }
                };
                
                // 添加合理性检查：根据代币类型区分处理
                let mut gas_limit = if is_eth {
                    // ETH转账的gas limit处理
                    if estimated_gas < U256::from(21_000) {
                        // ETH转账最小值为21000
                        println!("[DEBUG] ETH转账gas limit过小，使用最小值21000");
                        U256::from(21_000)
                    } else {
                        // 为估算值添加5%的安全边际
                        let gas_with_margin = estimated_gas * U256::from(105) / U256::from(100);
                        println!("[DEBUG] ETH转账添加5%安全边际: {} -> {}", estimated_gas, gas_with_margin);
                        gas_with_margin
                    }
                } else {
                    // 代币转账的gas limit处理
                    let chain_id = provider.get_chainid().await.unwrap_or_default().as_u64();
                    let min_token_gas = match chain_id {
                        42161 => U256::from(80_000),  // Arbitrum One - 更高的最小值
                        1 => U256::from(60_000),      // Ethereum Mainnet
                        137 => U256::from(60_000),    // Polygon
                        56 => U256::from(60_000),     // BSC
                        _ => U256::from(80_000),      // 其他链使用保守值
                    };
                    println!("[DEBUG] 链ID: {}, 代币转账最小Gas Limit: {}", chain_id, min_token_gas);
                    
                    if estimated_gas < min_token_gas {
                        // 代币转账gas limit过小，直接使用最小值
                        println!("[DEBUG] 估算的gas limit {} 小于最小值 {}，使用最小值", estimated_gas, min_token_gas);
                        min_token_gas
                    } else {
                        // 为估算值添加20%的安全边际，但确保不低于最小值
                        let gas_with_margin = estimated_gas * U256::from(120) / U256::from(100);
                        let final_gas = std::cmp::max(gas_with_margin, min_token_gas);
                        println!("[DEBUG] 代币转账添加20%安全边际: {} -> {} (最终: {})", estimated_gas, gas_with_margin, final_gas);
                        final_gas
                    }
                };
                
                // 关键检查：确保gas limit不超过区块限制
                if gas_limit > max_safe_gas_limit {
                    println!("[WARN] 估算的gas limit {} 超过区块安全上限 {}，调整为安全上限", gas_limit, max_safe_gas_limit);
                    gas_limit = max_safe_gas_limit;
                }
                
                // 最终验证：确保gas limit在合理范围内
                if gas_limit > block_gas_limit {
                    println!("[ERROR] gas limit {} 仍然超过区块限制 {}，强制设为区块限制的70%", gas_limit, block_gas_limit);
                    gas_limit = block_gas_limit * U256::from(70) / U256::from(100);
                }
                
                println!("[INFO] 最终确定的gas limit: {}", gas_limit);
                Ok(gas_limit)
            }
            "2" => {
                // 使用固定Gas Limit，但仍需检查是否超过区块限制
                let fixed_gas_limit = U256::from(config.limit_count);
                if fixed_gas_limit > max_safe_gas_limit {
                    println!("[WARN] 固定gas limit {} 超过区块安全上限 {}，调整为安全上限", fixed_gas_limit, max_safe_gas_limit);
                    Ok(max_safe_gas_limit)
                } else {
                    Ok(fixed_gas_limit)
                }
            }
            "3" => {
                // 使用随机Gas Limit，但仍需检查是否超过区块限制
                let mut rng = rand::thread_rng();
                let random_gas_limit = rng.gen_range(config.limit_count_list[0]..=config.limit_count_list[1]);
                let gas_limit = U256::from(random_gas_limit);
                
                if gas_limit > max_safe_gas_limit {
                    println!("[WARN] 随机gas limit {} 超过区块安全上限 {}，调整为安全上限", gas_limit, max_safe_gas_limit);
                    Ok(max_safe_gas_limit)
                } else {
                    Ok(gas_limit)
                }
            }
            _ => Err("gas limit type error".to_string()),
        }
    }
}

// Tauri命令：基础币转账
#[tauri::command]
pub async fn base_coin_transfer<R: tauri::Runtime>(
    app_handle: tauri::AppHandle<R>,
    index: usize,
    item: TransferItem,
    config: TransferConfig,
) -> Result<TransferResult, String> {
    match base_coin_transfer_internal(app_handle, index, item, config).await {
        Ok(tx_hash) => Ok(TransferResult {
            success: true,
            tx_hash: Some(tx_hash),
            error: None,
        }),
        Err(e) => Ok(TransferResult {
            success: false,
            tx_hash: None,
            error: Some(e.to_string()),
        }),
    }
}

// 内部基础币转账实现
async fn base_coin_transfer_internal<R: tauri::Runtime>(
    app_handle: tauri::AppHandle<R>,
    index: usize,
    mut item: TransferItem,
    config: TransferConfig,
) -> Result<String, Box<dyn std::error::Error>> {
    item.retry_flag = false;
    
    // 不再在方法开头创建固定的provider，改为在每次RPC调用时动态获取
    
    // 创建钱包
    if item.private_key.trim().is_empty() {
        return Err("私钥不能为空！".into());
    }
    
    // 处理私钥格式，兼容带0x和不带0x的格式
    let private_key = if item.private_key.starts_with("0x") || item.private_key.starts_with("0X") {
        item.private_key[2..].to_string()
    } else {
        item.private_key.clone()
    };
    
    let wallet = private_key.parse::<LocalWallet>().map_err(|e| {
        format!("私钥格式错误: {}，请检查私钥格式是否正确（应为64位十六进制字符串，可带或不带0x前缀）", e)
    })?;
    // 优先通过统一ProviderUtils获取链ID，避免重复查询逻辑
    let chain_id = match ProviderUtils::get_chain_id(&config.chain).await {
        Ok(id) => id,
        Err(_) => {
            // 如果ProviderUtils获取失败，尝试从RPC配置获取
            match get_rpc_config(&config.chain).await {
                Some(c) => c.chain_id,
                None => {
                    return Err(format!(
                        "无法获取链 '{}' 的配置信息。请检查：1) 链是否存在  2) 是否至少有一个启用的RPC节点。",
                        config.chain
                    ).into());
                }
            }
        }
    };
    let wallet = wallet.with_chain_id(chain_id);
    let wallet_address = wallet.address();
    
    // 解析目标地址
    if item.to_addr.trim().is_empty() {
        return Err("目标地址不能为空，请先导入接收地址！".into());
    }
    let to_address: Address = item.to_addr.parse().map_err(|e| {
        format!("目标地址格式错误: {}，请检查地址格式是否正确", e)
    })?;
    
    // 获取当前使用的RPC URL用于错误信息
    let rpc_url = if let Some(rpc_config) = get_rpc_config(&config.chain).await {
        match rpc_config.get_random_rpc() {
            Ok(url) => url.to_string(),
            Err(e) => {
                return Err(format!("获取RPC地址失败: {}", e).into());
            }
        }
    } else {
        return Err(format!(
            "无法获取链 '{}' 的RPC配置。请在RPC管理中至少启用一个RPC节点。",
            config.chain
        ).into());
    };
    
    // 预检查余额是否充足（避免RPC调用后才发现余额不足）
    let provider_for_precheck = get_random_provider(&config.chain).await.map_err(|e| {
        format!("获取RPC提供商失败: {}", e)
    })?;
    TransferUtils::pre_check_balance(&config, provider_for_precheck.clone(), wallet_address, to_address).await.map_err(|e| {
        format!("余额预检查失败 (RPC: {}): {}", rpc_url, e)
    })?;
    
    // 获取余额
    let provider_for_balance = get_random_provider(&config.chain).await.map_err(|e| {
        format!("获取RPC提供商失败: {}", e)
    })?;
    let balance = provider_for_balance.get_balance(wallet_address, None).await.map_err(|e| {
        format!("获取余额失败 (RPC: {}): {}", rpc_url, e)
    })?;
    let balance_ether_str = format_ether(balance);
    
    println!("序号：{}, 当前余额为: {} ETH", index, balance_ether_str);
    
    // 获取Gas Price
    let provider_for_gas_price = get_random_provider(&config.chain).await.map_err(|e| {
        format!("获取RPC提供商失败: {}", e)
    })?;
    
    let gas_price = TransferUtils::get_gas_price(&config, provider_for_gas_price.clone()).await.map_err(|e| {
        format!("获取Gas Price失败 (RPC: {}): {}", rpc_url, e)
    })?;
    

    
    // 检查gas_price是否为0
    if gas_price.is_zero() {
        return Err("获取到的 gas price 为0，请检查网络连接或RPC配置".into());
    }
    
    // 获取Gas Limit - 根据用户设置直接获取，避免不必要的网络调用
    let mut gas_limit = match config.limit_type.as_str() {
        "1" => {
            // 自动估算模式才需要网络调用
            // 对于全部转账，需要使用实际的转账金额来估算gas limit
            if config.transfer_type == "1" {
                // 全部转账：使用多层回退机制估算gas limit
                // 1. 首先尝试用余额的90%估算
                let amount_90_percent = balance * U256::from(90) / U256::from(100);
                let provider_for_gas_limit_90 = get_random_provider(&config.chain).await?;
                match TransferUtils::get_gas_limit(
                     &config,
                     provider_for_gas_limit_90.clone(),
                     wallet_address,
                     to_address,
                     amount_90_percent,
                 ).await {
                     Ok(gas_limit) => gas_limit,
                     Err(_) => {
                         println!("序号：{}, 90%余额估算gas limit失败，尝试0.001 ETH估算", index);
                         // 2. 如果90%估算失败，尝试用0.001 ETH估算
                         let fallback_amount = parse_ether(0.000003).map_err(|e| format!("解析金额失败: {}", e))?;
                         let provider_for_gas_limit_fallback = get_random_provider(&config.chain).await?;
                         match TransferUtils::get_gas_limit(
                             &config,
                             provider_for_gas_limit_fallback.clone(),
                             wallet_address,
                             to_address,
                             fallback_amount,
                         ).await {
                             Ok(gas_limit) => gas_limit,
                             Err(_) => {
                                 println!("序号：{}, 0.000003 ETH估算gas limit也失败", index);
                                 // 3. 如果0.001 ETH也失败，返回余额不足错误
                                 return Err("当前余额不足支付Gas费用，不做转账操作！".into());
                             }
                         }
                     }
                 }
            } else {
                // 其他转账类型使用最小金额估算
                let estimate_amount = parse_ether(0.000003).map_err(|e| format!("解析金额失败: {}", e))?;
                let provider_for_gas_limit_estimate = get_random_provider(&config.chain).await?;
                TransferUtils::get_gas_limit(
                    &config,
                    provider_for_gas_limit_estimate.clone(),
                    wallet_address,
                    to_address,
                    estimate_amount,
                ).await.map_err(|e| format!("获取gas limit失败: {}", e))?
            }
        }
        "2" => {
            // 固定数量模式直接使用设定值
            U256::from(config.limit_count)
        }
        "3" => {
            // 随机范围模式生成随机值
            let mut rng = rand::thread_rng();
            let random_limit = rng.gen_range(config.limit_count_list[0]..=config.limit_count_list[1]);
            U256::from(random_limit)
        }
        _ => {
            return Err("gas limit type error".into());
        }
    };
    
    println!("序号：{}, gas limit: {}", index, gas_limit);

    // 计算转账金额
    let transfer_amount = match config.transfer_type.as_str() {
        "1" => {
            // 全部转账 - 使用多轮优化的二分法精确计算最大可转账金额
            let mut final_transfer_amount = U256::zero();
            let mut final_gas_limit = gas_limit;
            
            // 发送计算开始状态到前端
            let _ = app_handle.emit("transfer_status_update", serde_json::json!({
                "index": index - 1,
                "error_msg": "计算转账金额中...",
                "exec_status": "1"
            }));
            
            // 根据金额大小动态调整优化策略
            let balance_eth = ethers::utils::format_units(balance, 18)
                .unwrap_or_default()
                .parse::<f64>()
                .unwrap_or(0.0);
            
            let (safety_margins, search_ranges, max_iterations, amount_type): (Vec<i32>, Vec<i32>, usize, &str) = if balance_eth > 0.1 {
                // 大金额：3轮优化，大幅减少迭代次数提升速度
                (vec![103, 102, 101], vec![97, 98, 99], 12, "大金额")
            } else if balance_eth > 0.01 {
                // 中型金额：2轮优化
                (vec![102, 101], vec![98, 99], 10, "中型金额")
            } else if balance_eth > 0.001 {
                // 小型金额：2轮优化
                (vec![102, 101], vec![98, 99], 8, "小型金额")
            } else {
                // 极简金额：1轮快速计算
                (vec![101], vec![99], 6, "极简金额")
            };
            
            println!("序号：{}, 开始多轮二分法优化，余额: {} ETH ({})", index, balance_eth, amount_type);
            
            let mut no_improvement_count = 0; // 连续无改进计数器
            let mut last_best_amount = final_transfer_amount;
            
            for (round, (&margin, &range)) in safety_margins.iter().zip(search_ranges.iter()).enumerate() {
                // 发送当前轮次状态到前端
                let status_msg = match round {
                    0 => "转账金额第一轮优化中...",
                    1 => "转账金额第二轮优化中...", 
                    2 => "转账金额最终优化中...",
                    _ => "计算转账金额中..."
                };
                let _ = app_handle.emit("transfer_status_update", serde_json::json!({
                    "index": index - 1,
                    "error_msg": status_msg,
                    "exec_status": "1"
                }));
                
                println!("序号：{}, 第{}轮优化: 安全边际{}%, 搜索范围{}%", index, round + 1, margin - 100, range);
                
                let mut low = if round == 0 { U256::zero() } else { final_transfer_amount }; // 后续轮次从上一轮结果开始
                let mut high = balance * U256::from(range) / U256::from(100);
                let mut best_amount = final_transfer_amount;
                let mut best_gas_limit = final_gas_limit;
                
                // 使用动态迭代次数
                let mut cached_gas_limit = gas_limit; // 缓存gas limit减少RPC调用
                for iteration in 0..max_iterations {
                    if high <= low {
                        break;
                    }
                    
                    let mid = (low + high) / U256::from(2);
                    if mid == U256::zero() || mid <= best_amount {
                        break;
                    }
                    
                    // 优化：只在关键迭代位置进行gas估算，其他时候使用缓存值
                    let should_estimate_gas = iteration == 0 || // 第一次迭代
                                             iteration % 4 == 0 || // 每4次迭代
                                             iteration == max_iterations - 1; // 最后一次迭代
                    
                    // 估算这个转账金额需要的gas limit
                    let estimated_gas_limit = if config.limit_type == "1" && should_estimate_gas {
                        let provider_for_gas_estimation = get_random_provider(&config.chain).await?;
                        match TransferUtils::get_gas_limit(
                            &config,
                            provider_for_gas_estimation.clone(),
                            wallet_address,
                            to_address,
                            mid,
                        ).await {
                            Ok(gas) => {
                                // 使用当前轮次的安全边际
                                let gas_with_margin = gas * U256::from(margin) / U256::from(100);
                                cached_gas_limit = gas_with_margin; // 更新缓存
                                gas_with_margin
                            }
                            Err(_) => {
                                // 估算失败，使用默认值加安全边际
                                gas_limit * U256::from(margin) / U256::from(100)
                            }
                        }
                    } else {
                        // 非自动估算模式或使用缓存值
                        if config.limit_type == "1" {
                            cached_gas_limit // 使用缓存的gas limit
                        } else {
                            gas_limit * U256::from(margin) / U256::from(100)
                        }
                    };
                    
                    let total_cost = mid + (gas_price * estimated_gas_limit);
                    
                    if iteration % 5 == 0 || iteration < 3 {
                        println!("序号：{}, 第{}轮迭代{}: 尝试转账={}, gas_limit={}, 总成本={}", 
                            index, round + 1, iteration + 1, mid, estimated_gas_limit, total_cost);
                    }
                    
                    if total_cost <= balance {
                        // 这个金额可行，尝试更大的金额
                        best_amount = mid;
                        best_gas_limit = estimated_gas_limit;
                        low = mid + U256::from(1);
                    } else {
                        // 这个金额太大，尝试更小的金额
                        high = mid - U256::from(1);
                    }
                }
                
                // 如果这一轮找到了更好的结果
                if best_amount > final_transfer_amount {
                    // 发送验证状态到前端
                    let _ = app_handle.emit("transfer_status_update", serde_json::json!({
                        "index": index - 1,
                        "error_msg": "验证计算结果...",
                        "exec_status": "1"
                    }));
                    
                    // 进行精确验证
                    let verification_cost = best_amount + (gas_price * best_gas_limit);
                    if verification_cost <= balance {
                        final_transfer_amount = best_amount;
                        final_gas_limit = best_gas_limit;
                        println!("序号：{}, 第{}轮成功: 转账金额={}, gas_limit={}, 剩余={}", 
                            index, round + 1, final_transfer_amount, final_gas_limit, 
                            balance - verification_cost);
                        
                        // 检查是否有显著改进
                        if final_transfer_amount > last_best_amount {
                            no_improvement_count = 0; // 重置计数器
                            last_best_amount = final_transfer_amount;
                        } else {
                            no_improvement_count += 1;
                        }
                    } else {
                        println!("序号：{}, 第{}轮验证失败，保持上一轮结果", index, round + 1);
                        no_improvement_count += 1;
                        println!("序号：{}, 第{}轮验证失败，连续无改进次数: {}", index, round + 1, no_improvement_count);
                        break; // 验证失败，停止更激进的尝试
                    }
                } else {
                    no_improvement_count += 1;
                    println!("序号：{}, 第{}轮无改进，连续无改进次数: {}", index, round + 1, no_improvement_count);
                }
                
                // 收敛检测：连续2次无改进时提前退出（优化速度）
                if no_improvement_count >= 2 {
                    println!("序号：{}, 连续{}次无改进，提前结束优化", index, no_improvement_count);
                    break;
                }
                
                // 改进幅度检测：如果改进幅度小于0.1%，提前退出
                if final_transfer_amount > U256::zero() && last_best_amount > U256::zero() {
                    let improvement_ratio = ((final_transfer_amount - last_best_amount).as_u128() as f64) / (last_best_amount.as_u128() as f64);
                    if improvement_ratio < 0.001 { // 0.1%
                        println!("序号：{}, 改进幅度 {:.4}% 小于阈值，提前结束优化", index, improvement_ratio * 100.0);
                        break;
                    }
                }
            }
            
            // 最终微调优化：当剩余金额超过0.0001 ETH时才执行，否则跳过节省时间
            if final_transfer_amount > U256::zero() {
                let current_cost = final_transfer_amount + (gas_price * final_gas_limit);
                let remaining = balance - current_cost;
                let remaining_eth = ethers::utils::format_units(remaining, 18)
                    .unwrap_or_default()
                    .parse::<f64>()
                    .unwrap_or(0.0);
                
                // 只有当剩余金额 > 0.0001 ETH 时才进行微调，节省RPC调用
                if remaining_eth > 0.0001 {
                    // 发送最终微调状态到前端
                    let _ = app_handle.emit("transfer_status_update", serde_json::json!({
                        "index": index - 1,
                        "error_msg": "最终微调优化中...",
                        "exec_status": "1"
                    }));
                    
                    println!("序号：{}, 开始最终微调优化，剩余 {} ETH", index, remaining_eth);
                    
                    let additional = remaining * U256::from(90) / U256::from(100);
                    let new_transfer_amount = final_transfer_amount + additional;
                    
                    // 重新估算gas limit
                    if config.limit_type == "1" {
                        let provider_for_final_gas = get_random_provider(&config.chain).await?;
                        if let Ok(new_gas_limit) = TransferUtils::get_gas_limit(
                            &config,
                            provider_for_final_gas.clone(),
                            wallet_address,
                            to_address,
                            new_transfer_amount,
                        ).await {
                            let new_gas_with_margin = new_gas_limit * U256::from(101) / U256::from(100); // 1%安全边际
                            let new_total_cost = new_transfer_amount + (gas_price * new_gas_with_margin);
                            
                            if new_total_cost <= balance {
                                final_transfer_amount = new_transfer_amount;
                                final_gas_limit = new_gas_with_margin;
                                println!("序号：{}, 微调成功: 增加转账金额={}", index, additional);
                            }
                        }
                    }
                } else {
                    println!("序号：{}, 剩余金额 {} ETH 太小，跳过最终微调优化", index, remaining_eth);
                }
            }
            
            // 如果所有优化都失败，使用保守估算
            if final_transfer_amount == U256::zero() {
                println!("序号：{}, 所有优化失败，使用保守估算", index);
                let conservative_gas_fee = parse_ether(0.001).unwrap_or(U256::from(1000000000000000u64)); // 0.001 ETH
                if conservative_gas_fee >= balance {
                    return Err("当前余额不足支付Gas费用，不做转账操作！".into());
                }
                final_transfer_amount = balance - conservative_gas_fee;
                final_gas_limit = conservative_gas_fee / gas_price;
            }
            
            let final_gas_fee = gas_price * final_gas_limit;
            let final_remaining = balance - final_transfer_amount - final_gas_fee;
            println!("序号：{}, 最终优化结果: balance={}, transfer_amount={}, gas_fee={}, remaining={}", 
                index, balance, final_transfer_amount, final_gas_fee, final_remaining);
            
            // 发送计算完成状态到前端
            let _ = app_handle.emit("transfer_status_update", serde_json::json!({
                "index": index - 1,
                "error_msg": "转账金额计算完成",
                "exec_status": "1"
            }));
            
            // 更新gas_limit为最终计算的值
            gas_limit = final_gas_limit;
            final_transfer_amount
        }
        "2" => {
            // 转账固定数量
            let amount = parse_ether(config.transfer_amount)?;
            if amount >= balance {
                return Err("当前余额不足，不做转账操作！".into());
            }
            amount
        }
        "3" => {
            // 转账随机数量
            let mut rng = rand::thread_rng();
            let random_amount = rng.gen_range(config.transfer_amount_list[0]..=config.transfer_amount_list[1]);
            // 根据精度设置格式化随机金额
            let formatted_amount = format!("{:.precision$}", random_amount, precision = config.amount_precision as usize);
            let precise_amount: f64 = formatted_amount.parse()?;
            let amount = parse_ether(precise_amount)?;
            if amount >= balance {
                return Err("当前余额不足，不做转账操作！".into());
            }
            amount
        }
        "4" => {
            // 剩余随机数量
            // 使用format_units替代format_ether，确保精度不丢失
            let balance_ether_str = format_units(balance, 18).map_err(|e| {
                format!("余额格式化失败: {}, 原始余额: {}", e, balance)
            })?;
            let balance_f64: f64 = balance_ether_str.parse::<f64>().map_err(|e| {
                format!("余额转换失败: {}, 原始余额: {}, format_units结果: {}", e, balance, balance_ether_str)
            })?;
            

            
            // 使用之前已获取的gas_limit
            
            let gas_fee = gas_price * gas_limit;
            let gas_fee_ether_str = format_units(gas_fee, 18).map_err(|e| {
                format!("gas费用格式化失败: {}, gas_fee: {}", e, gas_fee)
            })?;
            let gas_fee_ether: f64 = gas_fee_ether_str.parse::<f64>().map_err(|e| {
                format!("gas费用转换失败: {}, gas_fee: {}, format_units结果: {}", e, gas_fee, gas_fee_ether_str)
            })?;
            

            
            // 可用于转账的余额 = 总余额 - Gas费用
            let available_balance = balance_f64 - gas_fee_ether;
            
            println!("序号：{}, 总余额: {}, Gas费用: {}, 可用余额: {}", index, balance_f64, gas_fee_ether, available_balance);
            
            // 检查可用余额是否足够满足最小剩余数量要求
            if available_balance <= config.left_amount_list[1] {
                return Err(format!(
                    "当前可用余额为：{} (总余额: {} - Gas费用: {})，无法满足最大剩余数量 {} 要求，不做转账操作！",
                    available_balance, balance_f64, gas_fee_ether, config.left_amount_list[1]
                ).into());
            }
            
            let mut rng = rand::thread_rng();
            let left_amount = rng.gen_range(config.left_amount_list[0]..=config.left_amount_list[1]);
            let transfer_amount_f64 = available_balance - left_amount;
            
            if transfer_amount_f64 <= 0.0 {
                return Err(format!(
                    "计算转账金额为负数或零：可用余额 {} - 剩余数量 {} = {}，不做转账操作！",
                    available_balance, left_amount, transfer_amount_f64
                ).into());
            }
            
            // 根据精度设置格式化转账金额
            let formatted_amount = format!("{:.precision$}", transfer_amount_f64, precision = config.amount_precision as usize);
            let precise_amount: f64 = formatted_amount.parse()?;
            
            println!("序号：{}, 剩余数量: {}, 转账金额: {} (格式化后: {})", index, left_amount, transfer_amount_f64, precise_amount);
            
            parse_ether(precise_amount)?
        }
        _ => return Err("无效的转账类型".into()),
    };
    
    println!("序号：{}, 转账数量为: {}", index, format_units(transfer_amount, 18).unwrap_or_else(|_| "0".to_string()));
    
    // 构建交易（使用之前已获取的gas_limit）
    let tx = TransactionRequest::new()
        .from(wallet_address)
        .to(to_address)
        .value(transfer_amount)
        .gas_price(gas_price)
        .gas(gas_limit);
    
    // 发送交易
    item.error_msg = "发送交易...".to_string();
    // 发送状态更新事件到前端
    let _ = app_handle.emit("transfer_status_update", serde_json::json!({
        "index": index - 1,
        "error_msg": item.error_msg.clone(),
        "exec_status": "1"
    }));
    
    let provider_for_transaction = get_random_provider(&config.chain).await.map_err(|e| {
        format!("获取RPC提供商失败: {}", e)
    })?;
    let client = SignerMiddleware::new(provider_for_transaction.clone(), wallet);
    let pending_tx = client.send_transaction(tx, None).await.map_err(|e| {
        format!("发送交易失败 (RPC: {}): {}", rpc_url, e)
    })?;
    
    let tx_hash = pending_tx.tx_hash();
    println!("序号：{}, 交易 hash 为：{:?}", index, tx_hash);
    
    // 等待交易确认（设置30秒超时）
    item.error_msg = "等待交易结果...".to_string();
    // 发送状态更新事件到前端
    let _ = app_handle.emit("transfer_status_update", serde_json::json!({
        "index": index - 1,
        "error_msg": item.error_msg.clone(),
        "exec_status": "1"
    }));
    
    println!("[DEBUG] 开始等待交易确认，设置30秒超时...");
    let receipt = match tokio::time::timeout(
        tokio::time::Duration::from_secs(30),
        pending_tx
    ).await {
        Ok(result) => {
            result.map_err(|e| {
                format!("等待交易确认失败 (RPC: {}): {}", rpc_url, e)
            })?
        }
        Err(_) => {
            let timeout_msg = format!("等待交易确认超时 (RPC: {}) - 超过30秒未收到确认，交易哈希: {:?}", rpc_url, tx_hash);
            println!("[ERROR] {}", timeout_msg);
            return Err(timeout_msg.into());
        }
    };
    
    match receipt {
        Some(receipt) => {
            if receipt.status == Some(U64::from(1)) {
                Ok(format!("{:?}", receipt.transaction_hash))
            } else {
                Err(format!("交易失败 (RPC: {})", rpc_url).into())
            }
        }
        None => {
            Err(format!("交易未确认 (RPC: {})", rpc_url).into())
        }
    }
}