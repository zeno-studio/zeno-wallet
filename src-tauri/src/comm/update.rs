// src/commands/update.rs
use tauri::{AppHandle, Runtime};
use reqwest::Client;
use sha2::{Sha256, Digest};
use ed25519_dalek::{Verifier, PublicKey};

#[tauri::command]
pub async fn check_update<R: Runtime>(
    app: AppHandle<R>,
    current_version: String,
) -> Result<UpdateInfo, String> {
    let client = Client::new();
    let resp: UpdateInfo = client
        .get("https://your-worker.workers.dev/check/latest")
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;

    // 版本检查
    if resp.version <= current_version {
        return Err("No update".into());
    }

    // 下载
    let mut file_data = client
        .get(&resp.url)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .bytes()
        .await
        .map_err(|e| e.to_string())?;

    // 验证哈希
    let hash = format!("{:x}", Sha256::digest(&file_data));
    if hash != resp.hash {
        return Err("Hash mismatch".into());
    }

    // 验证签名（可选，防伪造）
    let pubkey = PublicKey::from_bytes(&hex::decode("your-pubkey").unwrap()).unwrap();
    let sig = ed25519_dalek::Signature::from_bytes(&hex::decode(&resp.signature).unwrap()).unwrap();
    if pubkey.verify(&file_data, &sig).is_err() {
        return Err("Invalid signature".into());
    }

    // 保存到临时文件
    let path = app.path_resolver().app_local_data_dir().unwrap().join("update.tmp");
    std::fs::write(&path, file_data).map_err(|e| e.to_string())?;

    Ok(resp)
}