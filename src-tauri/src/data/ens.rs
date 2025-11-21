use helios::{rpc::Http, types::Address, ens::Ens};
use cached::proc_macro::cached;
use std::time::Duration;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

/// 用 cached 宏做缓存，TTL 10 分钟
#[cached(time = 600, result = true)]
async fn resolve_ens_helios(name: &str, rpc_url: &str) -> Result<Address> {
    // 创建 HTTP provider
    let client = Http::new(rpc_url)?;
    let ens = Ens::new(client);

    let address = ens.name(name).await?;
    Ok(address)
}

/// 反向解析 address -> ENS name
/// cached(time = 600, result = true) 表示缓存 10 分钟
#[cached(time = 600, result = true)]
async fn reverse_lookup_helios(address: Address, rpc_url: &str) -> Result<Option<String>> {
    let client = Http::new(rpc_url)?;
    let ens = Ens::new(client);

    let name = ens.reverse(address).await?;
    Ok(name)
}

