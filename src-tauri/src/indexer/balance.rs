// 查询基础币种余额
    async fn query_base_balance(&self, item: &mut QueryItem, chain: &str) -> Result<()> {
        let rpc_url = self.get_rpc_url(chain).await?;
        
        // 查询余额
        let balance_result = self.send_rpc_request(
            &rpc_url,
            "eth_getBalance",
            serde_json::json!([item.address, "latest"])
        ).await?;

        if let Some(balance_hex) = balance_result.as_str() {
            let hex_without_prefix = &balance_hex[2..];
            match u128::from_str_radix(hex_without_prefix, 16) {
                Ok(balance_wei) => {
                    let balance_eth = balance_wei as f64 / 1e18;
                    item.plat_balance = Some(format!("{:.6}", balance_eth));
                }
                Err(e) => {
                    return Err(anyhow!("余额数值转换失败: {} (原始值: {})", e, balance_hex));
                }
            }
        }

        Ok(())
    }