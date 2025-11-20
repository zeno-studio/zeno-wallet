// src/multicall3.rs
// 极简 Multicall3 工具，只依赖你的 EthRpcProvider + serde_json
// 适用于 99.9% 的链（地址统一：0xcA11bde05977b3631167028862bE2a173976CA11）

use crate::rpc::https::{EthRpcProvider, RpcMethod};
use serde_json::{json, Value};
use std::collections::HashMap;
use crate::error::AppError;

const MULTICALL3_ADDR: &str = "0xcA11bde05977b3631167028862bE2a173976CA11";

pub struct Multicall3 {
    rpc: EthRpcProvider,
}

impl Multicall3 {
    pub fn new(rpc: EthRpcProvider) -> Self {
        Self { rpc }
    }

    /// 最强一击：批量查询任意 view 函数（允许部分失败）
    /// calls: vec![(target_contract, calldata_hex)]
    pub async fn aggregate3(
        &self,
        calls: Vec<(String, String)>, // (target: "0x...", calldata: "0x1234...")
    ) -> Result<Vec<MulticallResult>, AppError> {
        let calls_json = calls
            .into_iter()
            .map(|(target, call_data)| {
                json!({
                    "target": target,
                    "allowFailure": true,
                    "callData": call_data
                })
            })
            .collect::<Vec<_>>();

        let payload = json!({
            "to": MULTICALL3_ADDR,
            "data": Self::encode_aggregate3(calls_json)?
        });

        let raw_result = self.rpc.call(RpcMethod::Custom("eth_call"), json!([payload, "latest"])).await?;

        // 返回格式：{"0": "0x...", "1": "0x..."} 或直接数组
        let results = if raw_result.is_object() {
            let obj = raw_result.as_object().unwrap();
            obj.values().cloned().collect::<Vec<_>>()
        } else if raw_result.is_array() {
            raw_result.as_array().unwrap().clone()
        } else {
            vec![raw_result]
        };

        // 解析 returnData
        let mut parsed = Vec::new();
        for data in results {
            let data_str = data.as_str().unwrap_or("0x");
            let return_data = if data_str.starts_with("0x") {
                data_str[2..].to_string()
            } else {
                data_str.to_string()
            };
            parsed.push(MulticallResult {
                success: !return_data.is_empty() && return_data != "0",
                return_data,
            });
        }

        Ok(parsed)
    }

    /// 编码 aggregate3 的 calldata（纯手工，不依赖 alloy）
    fn encode_aggregate3(calls: Vec<Value>) -> Result<String, AppError> {
        // aggregate3 选择器：0x04f0f6d3
        let mut data = "04f0f6d3".to_string();

        // 编码 Call3[]: (address,bool,bytes)[]
        let calls_hex = alloy_primitives::hex::encode(&alloy_sol_types::SolValue::abi_encode(&calls));
        data += &calls_hex;

        Ok(format!("0x{}", data))
    }

    // 下面几个最常用的快捷方法（直接返回解析好的值）

    pub async fn get_eth_balances(
        &self,
        addresses: Vec<String>,
    ) -> Result<HashMap<String, String>, AppError> {
        let calls = addresses
            .iter()
            .map(|addr| {
                let calldata = format!(
                    "0x70a08231000000000000000000000000{}",
                    &addr[2..]
                );
                (MULTICALL3_ADDR.to_string(), calldata)
            })
            .collect();

        let results = self.aggregate3(calls).await?;
        let mut balances = HashMap::new();
        for (addr, result) in addresses.iter().zip(results.iter()) {
            balances.insert(addr.clone(), result.return_data.clone());
        }
        Ok(balances)
    }

    pub async fn get_block_number(&self) -> Result<u64, AppError> {
        let calldata = "0x42cbb15c".to_string(); // getBlockNumber()
        let result = self
            .aggregate3(vec![(MULTICALL3_ADDR.to_string(), calldata)])
            .await?;
        let hex = result[0].return_data.strip_prefix("0x").unwrap_or(&result[0].return_data);
        Ok(u64::from_str_radix(hex, 16).unwrap_or(0))
    }
}

#[derive(Debug, Clone)]
pub struct MulticallResult {
    pub success: bool,
    pub return_data: String, // hex string without 0x
}


// // 在你的钱包逻辑里
// let rpc = EthRpcProvider::new("https://rpc.ankr.com/eth")?;
// let multicall = Multicall3::new(rpc);

// // 批量查 100 个地址的 ETH 余额（一次 RPC！）
// let addresses = vec!["0x1234...".to_string(), "0x5678...".to_string()];
// let balances_hex = multicall.get_eth_balances(addresses).await?;
// for (addr, balance_hex) in balances_hex {
//     let balance = U256::from_str_radix(&balance_hex, 16)? / 1e18;
//     println!("{}: {} ETH", addr, balance);
// }