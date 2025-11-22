
use serde::{Deserialize, Serialize};
use crate::evm::address::Address;
use crate::evm::assets::AssetsType;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Token {
    pub chain_id: u64,
    pub address: Address,
    pub name: String, 
    pub symbol: String, 
    pub decimals: u64, 
    pub logo_url: Option<String>,
    pub assets_type: Option<AssetsType>,
}
