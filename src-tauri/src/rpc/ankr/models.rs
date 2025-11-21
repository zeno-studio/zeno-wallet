use crate::data::{IntoInterNft, Nft,TransactionHistoryEntry,IntoInterTx};
use crate::evm::{AssetsType, IntoInterAsset};
use crate::utils::num::{str_to_u64, str_to_f64,str_to_u256};

use alloy_primitives::{Address, address};
use serde::{Deserialize, Serialize};
use serde_json::json;

pub enum AnkrBlockchain {
    Eth,
    Bsc,
    Polygon,
    Arbitrum,
    Base,
    Linea,
    EthSepolia,
}

impl AnkrBlockchain {
    pub fn as_str(&self) -> &'static str {
        match self {
            AnkrBlockchain::Eth => "eth",
            AnkrBlockchain::Bsc => "bsc",
            AnkrBlockchain::Polygon => "polygon",
            AnkrBlockchain::Arbitrum => "arbitrum",
            AnkrBlockchain::Base => "base",
            AnkrBlockchain::Linea => "linea",
            AnkrBlockchain::EthSepolia => "eth_sepolia",
        }
    }
    pub fn to_array() -> Vec<&'static str> {
        vec!["eth", "bsc", "polygon", "arbitrum", "base", "linea"]
    }
    pub fn to_array_testnet() -> Vec<&'static str> {
        vec![
            "eth",
            "bsc",
            "polygon",
            "arbitrum",
            "base",
            "linea",
            "eth_sepolia",
        ]
    }
    pub fn to_chain_id(&self) -> u64 {
        match self {
            AnkrBlockchain::Ethereum => 1,
            AnkrBlockchain::Bsc => 56,
            AnkrBlockchain::Polygon => 137,
            AnkrBlockchain::Arbitrum => 42161,
            AnkrBlockchain::Base => 8453,
            AnkrBlockchain::Linea => 59140,
            AnkrBlockchain::EthSepolia => 11155111,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Trait {
    pub trait_type: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AnkrNft {
    pub blockchain: AnkrBlockchain,
    pub name: String,
    pub token_id: String,
    pub token_url: String,
    pub image_url: String,
    pub collection_name: String,
    pub symbol: String,
    #[serde(rename = "contractType")]
    pub contract_type: NftContractType,
    #[serde(rename = "contractAddress")]
    pub contract_address: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quantity: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub traits: Option<Vec<Trait>>,
}

impl IntoInterNft for AnkrNft {
    fn into_inter(self) -> Nft {
        Nft {
            chain_id: self.blockchain.to_chain_id(),
            address: address!(self.contract_address),
            name: self.name,
            symbol: self.symbol,
            token_id: self.token_id,
            quantity: self.quantity,
            token_uri: self.image_url,
            collection: self.collection_name,
            assets_type: self.contract_type.into_inter(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NftContractType {
    ERC721,
    ERC1155,
    UNDEFINED,
}

impl IntoInterAsset for NftContractType {
    fn into_inter(self) -> AssetsType {
        match self {
            NftContractType::ERC721 => AssetsType::ERC721,
            NftContractType::ERC1155 => AssetsType::ERC1155,
            NftContractType::UNDEFINED => AssetsType::UNDEFINED,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnkrTransaction {
    // ====== 核心字段（必须保留）======
    pub hash: String,
    pub blockchain: String,
    #[serde(rename = "blockNumber")]
    pub block_number: String,
    pub from: String,
    #[serde(default)]
    pub to: Option<String>,
    pub value: String,
    #[serde(rename = "gasPrice")]
    pub gas_price: Option<String>,
    #[serde(rename = "gasUsed")]
    pub gas_used: Option<String>,
    pub status: Option<String>,
    pub timestamp: Option<String>,

    #[serde(default, skip, rename = "contractAddress")]
    pub contract_address: Option<String>,
    #[serde(default, skip, rename = "blockHash")]
    pub block_hash: Option<String>,
    #[serde(default, skip, rename = "transactionIndex")]
    pub transaction_index: Option<String>,
    #[serde(default, skip, rename = "cumulativeGasUsed")]
    pub cumulative_gas_used: Option<String>,
    #[serde(default, skip)]
    pub logs: Option<Vec<Log>>,
    #[serde(default, skip)]
    pub input: Option<String>,
    #[serde(default, skip)]
    pub v: Option<String>,
    #[serde(default, skip)]
    pub r: Option<String>,
    #[serde(default, skip)]
    pub s: Option<String>,
    #[serde(default, skip)]
    pub method: Option<Method>,
    #[serde(default, skip)]
    pub gas: Option<String>,
    #[serde(default, skip)]
    pub nonce: Option<String>,
    #[serde(default, skip)]
    pub r#type: Option<String>,
}

impl IntoInterTx for AnkrTransaction {
    fn into_inter(self) -> TransactionHistoryEntry {
        TransactionHistoryEntry {
            chain_id: self.blockchain.to_chain_id(),
            hash: self.hash.to_lowercase(),
            block_number: str_to_u64(&self.block_number),
            from: self.from.to_lowercase(),
            to: to.unwrap_or_default().to_lowercase(),
            value: self.value.and_then(|s| str_to_u256(s).ok()),
            gas_price: self.gas_price.and_then(|s| str_to_u256(s).ok()),
            gas_used: self.gas_used.and_then(|s| str_to_u256(s).ok()),
            status: self.status.filter(|s| s == "1").map(|_| "success".to_string()),
            timestamp: self.timestamp.and_then(|s| str_to_u64(s).ok()),
        }
    }
}


// ====================== Balance ======================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Balance {
    pub blockchain: AnkrBlockchain,
    pub token_name: String,
    pub token_symbol: String,
    pub token_decimals: u8,
    pub token_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contract_address: Option<String>,
    pub holder_address: String,
    pub balance: String,             // 通常是格式化后的可读数量
    pub balance_raw_integer: String, // 原始整数（未除以 decimals）
    pub balance_usd: String,
    pub token_price: String,
    pub thumbnail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncStatus {
    timestamp: u64,
    lag: String,
    status: String,
}

// ====================== Balance API ======================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetAccountBalanceRequest {
    pub blockchain: Vec<&str>, // 支持单个或数组，serde 会自动 flatten
    pub wallet_address: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub only_whitelisted: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub native_first: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_size: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sync_check: Option<bool>,
}
impl Default for GetAccountBalanceRequest {
    fn default() -> Self {
        Self {
            blockchain: ANKR_CHAINS.to_vec(),
            wallet_address: String::new(),
            only_whitelisted: None,
            native_first: None,
            page_token: None,
            page_size: None,
            sync_check: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetAccountBalanceReply {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_page_token: Option<String>,
    pub total_balance_usd: String,
    pub total_count: u64,
    pub assets: Vec<Balance>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sync_status: Option<SyncStatus>,
}

// ====================== NFTs API ======================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetNFTsByOwnerRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blockchain: Option<Vec<AnkrBlockchain>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter: Option<Vec<HashMap<String, Vec<String>>>>, // 对应 TS 中的 { [key: string]: string[] }[]
    pub wallet_address: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_size: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sync_check: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetNFTsByOwnerReply {
    pub owner: String,
    pub assets: Vec<Nft>,
    pub next_page_token: String, // TS 中没有 ?，所以这里必填
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sync_status: Option<SyncStatus>,
}

// 用于处理 "latest" | "earliest" | 数字 的特殊类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum BlockIdentifier {
    Number(u64),
    Latest,
    Earliest,
}

impl Default for BlockIdentifier {
    fn default() -> Self {
        BlockIdentifier::Latest
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetTransactionsByAddressRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from_block: Option<BlockIdentifier>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to_block: Option<BlockIdentifier>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from_timestamp: Option<BlockIdentifier>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to_timestamp: Option<BlockIdentifier>,
    pub blockchain: Vec<AnkrBlockchain>, // 这里是必填数组（TS 中也是必填）
    pub address: Vec<String>,            // 支持多个地址
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_size: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub desc_order: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_logs: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sync_check: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetTransactionsByAddressReply {
    pub transactions: Vec<Transaction>,
    pub next_page_token: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sync_status: Option<SyncStatus>,
}
