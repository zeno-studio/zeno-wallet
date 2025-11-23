
use serde::{Deserialize, Serialize};
use alloy_primitives::{Address,address};
use crate::evm::assets::AssetsType;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Nft {
    pub chain_id: u64,
    pub address: Address,  
    pub name: String, 
    pub symbol: String, 
    pub token_id: Option<u64>,
    pub quantity: Option<u64>,
    pub token_uri: Option<String>,
    pub collection: Option<String>,
    pub assets_type: Option<AssetsType>,
}
pub trait IntoInterNft {
    fn into_inter(self) -> Nft;
}
