use helios::client::{Client, ClientBuilder};
use helios::config::networks::{Network, ChainConfig}
use helios::types::BlockTag;
use std::sync::Arc;
use tokio::sync::Mutex;
use tauri::{
    plugin::{Builder, TauriPlugin},
    Runtime,
};
use crate::core::state::AppState;
use tauri::State;

pub type HeliosClient = Client;

// Initialize Helios client helper function
pub async fn init_helios() -> Result<Client, Box<dyn std::error::Error>> {
    // Hardcoded mainnet config - should be configurable in production
    // ⚠️ Replace with real RPC URLs (e.g., Alchemy, Infura, QuickNode)
    let consensus_rpc = "https://www.lightclientdata.org"; // Public Beacon node - replace with your own
    let execution_rpc = "https://eth-mainnet.g.alchemy.com/v2/YOUR_API_KEY";
    
    let client: Client = ClientBuilder::new()
        .network(Network::MAINNET)
        .consensus_rpc(consensus_rpc)
        .execution_rpc(execution_rpc)
        .build()?;
    
    // Start client (sync block headers)
    println!("Helios: Starting client...");
    client.start().await?;
    println!("Helios: Client synced and ready.");
    
    Ok(client)
}


#[tauri::command]
pub async fn switch_chain(state: State<'_, AppState>, chain: String) -> Result<(), String> {
    let supported = ["eth", "base", "linea"];
    if !supported.contains(&chain.as_str()) {
        return Err(format!("Helios 不支持该链: {}", chain));
    }

    // 记录当前链
    *state.current_chain.lock().await = chain.clone();

    let mut clients = state.helios_clients.lock().await;

    // 如果已经存在，直接复用（秒级切换）
    if clients.contains_key(&chain) {
        return Ok(());
    }

    // 否则新建实例（首次 2~4 秒，后续全部复用）
    let network = match chain.as_str() {
        "eth" => Network::Mainnet,
        "base" => Network::Base,
        "linea" => Network::LineaMainnet,
        _ => unreachable!(),
    };

    let config = ChainConfig {
    network,
    execution_rpc: execution_rpc_for_chain(chain),     // 你自己的公共/付费 RPC
    consensus_rpc: consensus_rpc_for_chain(chain),     // 必须按链区分！
    checkpoint: None, // 自动拉最新 finalized checkpoint
    ..Default::default()
};

    let client: HeliosClient<FileDB> = ClientBuilder::new()
        .config(config)
        .build()
        .map_err(|e| e.to_string())?;

    // 后台启动（不阻塞 UI）
    tokio::spawn(async move {
        let _ = client.start().await;
    });

    clients.insert(chain, client);
    Ok(())
}

fn consensus_rpc_for_chain(chain: &str) -> String {
    match chain {
        "eth"      => "https://www.lightclientdata.org",
        "base"     => "https://base.lightclientdata.org",
        "linea"    => "https://linea.lightclientdata.org",
        "optimism" => "https://optimism.lightclientdata.org",
        "arbitrum" => "https://arbitrum.lightclientdata.org",
        _ => "https://www.lightclientdata.org", // 默认降级到主网（不会用）
    }.to_string()
}

fn execution_rpc_for_chain(chain: &str) -> String {
    match chain {
        "eth"      => "https://ethereum.publicnode.com",
        "base"     => "https://base.publicnode.com",
        "linea"    => "https://linea.publicnode.com",
        "optimism" => "https://optimism.publicnode.com",
        "arbitrum" => "https://arbitrum.publicnode.com",
        _ => "https://ethereum.publicnode.com",
    }.to_string()
}
