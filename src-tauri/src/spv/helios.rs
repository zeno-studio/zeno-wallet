// use ethers::types::H160;
// use revm::primitives::{AccountInfo, Bytecode, U256};
// use serde::{Deserialize, Serialize};
// use serde_json::json;
// use std::collections::HashMap;
// use std::{str::FromStr, sync::Arc};
// use tokio::sync::Mutex;

// #[derive(Debug, Clone, Deserialize, Serialize)]
// pub enum ProviderKind {
//     Public,
//     Custom(String),
//     WalletNode,
// }

// /// Minimal JSON-RPC client wrapper used to call remote node methods such as `eth_getProof`, `eth_getCode`, `eth_getStorageAt`, `eth_getBlockByNumber`.
// /// We intentionally use a simple HTTP JSON-RPC call here so the implementation is robust across helios versions and other providers.
// #[derive(Clone)]
// pub struct JsonRpcClient {
//     pub url: String,
//     pub client: reqwest::Client,
// }

// impl JsonRpcClient {
//     pub fn new(url: String) -> Self {
//         Self {
//             url,
//             client: reqwest::Client::new(),
//         }
//     }

//     pub async fn call<T: for<'de> serde::Deserialize<'de>>(
//         &self,
//         method: &str,
//         params: serde_json::Value,
//     ) -> Result<T, String> {
//         let payload = json!({"jsonrpc":"2.0","id":1,"method":method,"params":params});
//         let res = self
//             .client
//             .post(&self.url)
//             .json(&payload)
//             .send()
//             .await
//             .map_err(|e| e.to_string())?;
//         let status = res.status();
//         let text = res.text().await.map_err(|e| e.to_string())?;
//         if !status.is_success() {
//             return Err(format!("rpc error {}: {}", status, text));
//         }
//         let v: serde_json::Value = serde_json::from_str(&text).map_err(|e| e.to_string())?;
//         if let Some(err) = v.get("error") {
//             return Err(format!("rpc returned error: {}", err));
//         }
//         let result = v.get("result").ok_or_else(|| "no result".to_string())?;
//         serde_json::from_value(result.clone()).map_err(|e| e.to_string())
//     }
// }

// /// The HeliosClient in this module is a thin wrapper providing:
// pub fn parse_u256_hex(&self, s: &str) -> Result<U256, String> {
//     let raw = s.trim_start_matches("0x");
//     U256::from_str_radix(raw, 16).map_err(|e| e.to_string())
// }

// pub fn parse_bytes(&self, s: &str) -> Result<Vec<u8>, String> {
//     hex::decode(s.trim_start_matches("0x")).map_err(|e| e.to_string())
// }

// /// Fetch code for address and cache it. Uses eth_getCode.
// pub async fn get_code(&self, address: &str, block: Option<&str>) -> Result<Vec<u8>, String> {
//     // check cache first
//     {
//         let cache = self.code_cache.lock().await;
//         if let Some(b) = cache.get(address) {
//             return Ok(b.clone());
//         }
//     }
//     let params = if let Some(b) = block {
//         json!([address, b])
//     } else {
//         json!([address, "latest"])
//     };
//     let code_hex: String = self.rpc.call("eth_getCode", params).await?;
//     let bytes = hex::decode(code_hex.trim_start_matches("0x")).map_err(|e| e.to_string())?;
//     // cache
//     let mut cache = self.code_cache.lock().await;
//     cache.insert(address.to_string(), bytes.clone());
//     Ok(bytes)
// }

// /// Fetch a storage slot via eth_getStorageAt
// pub async fn get_storage_at(
//     &self,
//     address: &str,
//     slot_index_hex: &str,
//     block: Option<&str>,
// ) -> Result<Vec<u8>, String> {
//     let block_tag = block.unwrap_or("latest");
//     let params = json!([address, slot_index_hex, block_tag]);
//     let val_hex: String = self.rpc.call("eth_getStorageAt", params).await?;
//     let bytes = hex::decode(val_hex.trim_start_matches("0x")).map_err(|e| e.to_string())?;
//     Ok(bytes)
// }

// /// Fetch account proof via eth_getProof (returns JSON structure). We will not parse it fully here,
// /// but return the raw JSON so a Helios proof verification routine can operate on it.
// pub async fn get_proof(
//     &self,
//     address: &str,
//     storage_keys: Vec<String>,
//     block: Option<&str>,
// ) -> Result<serde_json::Value, String> {
//     let block_tag = block.unwrap_or("latest");
//     let params = json!([address, storage_keys, block_tag]);
//     let v: serde_json::Value = self.rpc.call("eth_getProof", params).await?;
//     Ok(v)
// }

// /// Fetch block header by number or tag
// pub async fn get_block_by_number(&self, tag: &str) -> Result<serde_json::Value, String> {
//     let params = json!([tag, false]);
//     let v: serde_json::Value = self.rpc.call("eth_getBlockByNumber", params).await?;
//     Ok(v)
// }

// /// Verify proof with Helios (placeholder).
// /// In a real integration you'd call into Helios's proof verification routines (crate helios::proofs::verify...)
// pub fn verify_proof_locally(&self, _proof_json: &serde_json::Value) -> Result<bool, String> {
//     // TODO: call Helios library APIs to validate the proof against known headers.
//     // For now we return Ok(true) as a placeholder.
//     Ok(true)
// }

// // Small DTO used to build the REVM DB overlay
// #[derive(Debug, Serialize, Deserialize, Clone)]
// pub struct VerifiedAccount {
//     pub address: String,
//     pub balance_hex: String,
//     pub nonce: u64,
//     pub code_hex: Option<String>,
//     pub storage: Option<HashMap<String, String>>,
// }

// #[derive(Debug, Serialize, Deserialize, Clone)]
// pub struct VerifiedState {
//     pub block_number: u64,
//     pub block_hash: String,
//     pub accounts: Vec<VerifiedAccount>,
// }

// impl HeliosClient {
//     /// Build a VerifiedState for a set of addresses required by a tx.
//     /// This function will:
//     /// 1. Call eth_getProof for the addresses and requested slots
//     /// 2. Verify proofs locally via Helios (TODO)
//     /// 3. Construct VerifiedState including code and storage
//     pub async fn build_verified_state_for_addresses(
//         &self,
//         addresses: Vec<String>,
//         slots: Vec<Vec<String>>,
//         block_tag: Option<&str>,
//     ) -> Result<VerifiedState, String> {
//         // Note: slots is a vector with same length as addresses where each element is a vec of slot-keys hex strings
//         let mut accounts = Vec::new();
//         for (i, addr) in addresses.iter().enumerate() {
//             let storage_keys = slots.get(i).cloned().unwrap_or_default();
//             let proof = self
//                 .get_proof(addr, storage_keys.clone(), block_tag)
//                 .await?;
//             // verify
//             let ok = self.verify_proof_locally(&proof)?;
//             if !ok {
//                 return Err("proof verification failed".into());
//             }
//             // parse out fields from proof result
//             let balance_hex = proof
//                 .get("balance")
//                 .and_then(|v| v.as_str())
//                 .unwrap_or("0x0")
//                 .to_string();
//             let nonce = proof
//                 .get("nonce")
//                 .and_then(|v| v.as_str())
//                 .and_then(|s| u64::from_str_radix(s.trim_start_matches("0x"), 16).ok())
//                 .unwrap_or(0);
//             let code_hex = proof.get("codeHash").and_then(|_| {
//                 // fetch code separately
//                 None
//             });
//             // fetch actual code
//             let code_bytes = self.get_code(addr, block_tag).await.ok();
//             let code_hex_final = code_bytes.as_ref().map(|b| format!("0x{}", hex::encode(b)));
//             // storage map
//             let mut storage_map = None;
//             if !storage_keys.is_empty() {
//                 let mut map = HashMap::new();
//                 for key in storage_keys.iter() {
//                     if let Ok(vb) = self.get_storage_at(addr, key, block_tag).await {
//                         map.insert(key.clone(), format!("0x{}", hex::encode(&vb)));
//                     }
//                 }
//                 storage_map = Some(map);
//             }
//             accounts.push(VerifiedAccount {
//                 address: addr.clone(),
//                 balance_hex,
//                 nonce,
//                 code_hex: code_hex_final,
//                 storage: storage_map,
//             });
//         }
//         // get some block info
//         let block = self
//             .get_block_by_number(block_tag.unwrap_or("latest"))
//             .await?;
//         let block_number = block
//             .get("number")
//             .and_then(|v| v.as_str())
//             .and_then(|s| u64::from_str_radix(s.trim_start_matches("0x"), 16).ok())
//             .unwrap_or(0);
//         let block_hash = block
//             .get("hash")
//             .and_then(|v| v.as_str())
//             .unwrap_or_default()
//             .to_string();
//         Ok(VerifiedState {
//             block_number,
//             block_hash,
//             accounts,
//         })
//     }
// }
