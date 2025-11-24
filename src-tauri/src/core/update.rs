
use alloy_sol_types::sol;
use alloy_primitives::{address, Address};
use alloy_provider::{Provider, ProviderBuilder};
use alloy_transport_http::Http;
use reqwest::Client;
use serde::Deserialize;
use std::time::{SystemTime, UNIX_EPOCH};

// 合约地址（部署后填这里，建议写死在代码里）
const UPDATER_ADDR: Address = address!("0xYourContractOnBaseOrEth");

// 用 alloy 的 sol! 宏生成类型安全的合约接口
sol! {
    #[sol(rpc)]
    contract WalletManifestPointer {
        function getManifest() external view returns (string memory manifestArweaveTx, uint256 updatedAt);
        function update(string calldata newTxId) external;
    }
}

// manifest.json 的结构（Arweave 上永久存储）
#[derive(Deserialize, Debug)]
struct Manifest {
    version: String,
    #[serde(rename = "releasedAt")]
    released_at: String,
    platforms: std::collections::HashMap<String, String>, // key: "windows" / "linux" / "macos" / "ios" / "android"
    #[serde(default)]
    signature: Option<String>,
}

// Helios 作为 Provider（你已经在用，就直接传进来）
async fn get_provider() -> impl Provider {
    // 例子：用 Helios 本地 RPC（127.0.0.1:8545）
    ProviderBuilder::new()
        .on_http("http://127.0.0.1:8545".parse().unwrap())
        .await
        .unwrap()
}

#[tauri::command]
pub async fn check_update() -> Result<String, String> {
    let provider = get_provider().await;
    let contract = WalletManifestPointer::new(UPDATER_ADDR, provider);

    // 1. 读取链上最新的 manifest Arweave tx_id
    let WalletManifestPointer::getManifestReturn { manifestArweaveTx: tx_id, updatedAt: ts } =
        contract.getManifest().call().await.map_err(|e| e.to_string())?;

    if tx_id.is_empty() {
        return Err("No manifest published yet".to_string());
    }

    // 2. 1 分钟延迟确认（Base ≈2s block，30 blocks ≈ 1 min）
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    if now.saturating_sub(ts) < 60 {
        return Ok("Update too recent, waiting for finality".to_string());
    }

    // 3. 从 Arweave 拉 manifest.json
    let manifest_url = format!("https://arweave.net/{}", tx_id);
    let manifest_json = Client::new()
        .get(&manifest_url)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .text()
        .await
        .map_err(|e| e.to_string())?;

    let manifest: Manifest = serde_json::from_str(&manifest_json).map_err(|e| e.to_string())?;

    // 4. 根据当前平台取对应的 Arweave 包地址
    let platform_key = if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "linux") {
        "linux"
    } else if cfg!(target_os = "macos") {
        "macos"
    } else if cfg!(target_os = "ios") {
        "ios"
    } else if cfg!(target_os = "android") {
        "android"
    } else {
        "unknown"
    };

    let package_tx = manifest
        .platforms
        .get(platform_key)
        .ok_or(format!("No package for {}", platform_key))?
        .clone();

    // 5. 返回给前端，让用户选择更新（或自动更新）
    Ok(format!(
        "New version {} available!\nDownload: https://arweave.net/{}",
        manifest.version, package_tx
    ))
}