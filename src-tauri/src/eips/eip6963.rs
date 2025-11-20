use serde::{Deserialize, Serialize};
use tauri::Manager;
use std::collections::HashMap;

/// EIP-6963 Provider Info structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EIP6963ProviderInfo {
    /// Unique identifier for the provider
    pub uuid: String,
    /// Human-readable name of the wallet
    pub name: String,
    /// URI-encoded icon for the wallet
    pub icon: String,
    /// Reverse DNS identifier for the wallet
    pub rdns: String,
}

/// EIP-6963 Provider Detail structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EIP6963ProviderDetail {
    /// Provider information
    pub info: EIP6963ProviderInfo,
    /// The actual provider object (not serialized)
    #[serde(skip)]
    pub provider: Option<String>, // In Rust context, we'll store a reference or identifier
}

impl EIP6963ProviderDetail {
    /// Create a new provider detail for Zeno Wallet
    pub fn new_zeno_wallet() -> Self {
        Self {
            info: EIP6963ProviderInfo {
                uuid: format!("zeno-wallet-{}", uuid::Uuid::new_v4()),
                name: "Zeno Wallet".to_string(),
                icon: "data:image/svg+xml;base64,PHN2ZyB3aWR0aD0iMjQiIGhlaWdodD0iMjQiIHZpZXdCb3g9IjAgMCAyNCAyNCIgZmlsbD0ibm9uZSIgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIj4KPHJlY3Qgd2lkdGg9IjI0IiBoZWlnaHQ9IjI0IiBmaWxsPSJ3aGl0ZSIvPgo8cGF0aCBkPSJNMTIgMkM2LjQ4IDAgMiA0LjQ4IDIgMTBDMiAxNS41MiA2LjQ4IDIwIDEyIDIwQzE3LjUyIDIwIDIyIDE1LjUyIDIyIDEwQzIyIDQuNDggMTcuNTIgMCAxMiAyWk0xMCAxN0w1IDEyTDEwIDdMMTAgMTJaTTE0IDE3TDE5IDEyTDE0IDdMMTQgMTJaIiBmaWxsPSIjMzMzMzMzIi8+Cjwvc3ZnPg==".to_string(),
                rdns: "com.zenowallet".to_string(),
            },
            provider: None,
        }
    }
}

/// Handle EIP-6963 provider announcement
#[tauri::command]
pub async fn eip6963_announce_provider(
    window: tauri::Window,
) -> Result<(), String> {
    let provider_detail = EIP6963ProviderDetail::new_zeno_wallet();
    
    // Emit the provider announcement event
    window.emit("eip6963:announce-provider", provider_detail)
        .map_err(|e| format!("Failed to announce provider: {}", e))?;
    
    Ok(())
}

/// Handle EIP-6963 provider request
#[tauri::command]
pub async fn eip6963_request_provider(
    window: tauri::Window,
) -> Result<(), String> {
    // When a DApp requests providers, we announce ours
    eip6963_announce_provider(window).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_info_creation() {
        let provider = EIP6963ProviderDetail::new_zeno_wallet();
        assert_eq!(provider.info.name, "Zeno Wallet");
        assert_eq!(provider.info.rdns, "com.zenowallet");
        assert!(provider.info.uuid.starts_with("zeno-wallet-"));
    }
}