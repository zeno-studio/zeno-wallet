

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Nft {
    pub chain_id: u64,
    pub address: String,  // HexString
    pub name: String,  // HexString
    pub symbol: String,  // HexString
    pub token_id: Option<u64>,
    pub quantity: Option<u64>,
    pub token_uri: Option<String>,
}