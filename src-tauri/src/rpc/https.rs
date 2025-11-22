// src-tauri/src/rpc/https.rs
use crate::error::AppError;
use reqwest::Client;
use std::time::Duration;

pub fn create_https_client() -> Result<Client, AppError> {
    Client::builder()
        .use_rustls_tls()
        .pool_max_idle_per_host(10)
        .http2_keep_alive_timeout(Duration::from_secs(30))
        .timeout(Duration::from_secs(10))
        .gzip(true)
        .brotli(true)
        .build()
        .map_err(|e| {
            AppError::ReqwestClientBuildError(e)
        })?;
}
