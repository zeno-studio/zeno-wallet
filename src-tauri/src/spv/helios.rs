use helios::client::ClientBuilder;
use helios::config::networks::{Network, Mainnet, Base};  // Base 是 OP Stack 示例
use alloy_primitives::Address;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Mainnet（Ethereum）
    let mainnet_client = ClientBuilder::new()
        .network(Mainnet)
        .build()?;
    mainnet_client.start().await?;

    // 2. OP Stack 示例（Base）
    let base_client = ClientBuilder::new()
        .network(Base)  // 自动 OP Stack 支持
        .build()?;
    base_client.start().await?;

    // 3. 通用查询（你的 5 链）
    let client = mainnet_client;  // 或切换到 base_client
    let addr: Address = "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045".parse()?;
    let balance = client.get_balance(&addr, None).await?;
    println!("Ethereum Balance: {} wei", balance);

    Ok(())
}