use crate::data::{IntoInterNft, IntoInterToken, IntoInterTx, Nft, Token, TransactionHistoryEntry};
use crate::evm::assets::{AssetsType, IntoInterAsset,mapper_assets_type};

use crate::utils::num::{str_to_f64, str_to_u64, str_to_u256};

use alloy_primitives::{Address, address};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::str::FromStr;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Log {
    pub address: String,
    pub topics: Vec<String>,
    pub data: String,
    #[serde(rename = "blockNumber")]
    pub block_number: String,
    #[serde(rename = "transactionHash")]
    pub transaction_hash: String,
    #[serde(rename = "transactionIndex")]
    pub transaction_index: String,
    #[serde(rename = "blockHash")]
    pub block_hash: String,
    #[serde(rename = "logIndex")]
    pub log_index: String,
    #[serde(rename = "removed")]
    pub removed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Method {
    pub name: String,
    pub inputs: Vec<MethodInput>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MethodInput {
    pub name: String,
    #[serde(rename = "type")]
    pub type_: String,
}

pub enum AnkrBlockchain {
    Eth,
    Optimism,
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
            AnkrBlockchain::Optimism => "optimism",
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
            "optimism",
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
            AnkrBlockchain::Eth => 1,
            AnkrBlockchain::Optimism => 10,
            AnkrBlockchain::Bsc => 56,
            AnkrBlockchain::Polygon => 137,
            AnkrBlockchain::Arbitrum => 42161,
            AnkrBlockchain::Base => 8453,
            AnkrBlockchain::Linea => 59140,
            AnkrBlockchain::EthSepolia => 11155111,
        }
    }
}

// ====================== Balance ======================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AnkrBalance {
    pub blockchain: AnkrBlockchain,
    #[serde(rename = "tokenName")]
    pub token_name: String,
    #[serde(rename = "tokenSymbol")]
    pub token_symbol: String,
    #[serde(rename = "tokenDecimals")]
    pub token_decimals: u8,
    #[serde(rename = "tokenType")]
    pub token_type: String,
    #[serde(skip_serializing_if = "Option::is_none", rename = "contractAddress")]
    pub contract_address: Option<String>,
    #[serde(default, skip, rename = "holderAddress")]
    pub holder_address: String,
    pub balance: String,
    #[serde(default, skip, rename = "balanceRawInteger")]
    pub balance_raw_integer: String,
    #[serde(default, skip, rename = "balanceUsd")]
    pub balance_usd: String,
    #[serde(rename = "tokenPrice")]
    pub token_price: String,
    pub thumbnail: String,
}

impl IntoInterToken for AnkrBalance {
    fn into_inter(self) -> Token {
        let logo_url = self.thumbnail;
        let assets_type = self.token_type.as_ref().map(|token_type| mapper_assets_type(token_type.clone()));
        let contract_address = self.contract_address.as_ref().map(|addr| Address::from_str(addr).unwrap_or(Address::ZERO));
        
        Token {
            chain_id: self.blockchain.to_chain_id(),
            address: contract_address.unwrap_or(Address::ZERO),
            name: self.token_name,
            symbol: self.token_symbol,
            decimals: self.token_decimals,
            logo_url: logo_url,
            assets_type: assets_type,
            contract_address: contract_address
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SyncStatus {
    pub timestamp: u64,
    pub lag: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GetAccountBalanceReply {
    #[serde(skip_serializing_if = "Option::is_none", rename = "nextPageToken")]
    pub next_page_token: Option<String>,
    #[serde(default, skip, rename = "totalBalanceUsd")]
    pub total_balance_usd: String,
    #[serde(default, skip, rename = "totalCount")]
    pub total_count: u64,
    pub assets: Vec<AnkrBalance>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "syncStatus")]
    pub sync_status: Option<SyncStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GetAccountBalanceRequest {
    pub blockchain: Vec<AnkrBlockchain>,
    #[serde(rename = "walletAddress")]
    pub wallet_address: String,
    #[serde(skip_serializing_if = "Option::is_none", rename = "onlyWhitelisted")]
    pub only_whitelisted: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "nativeFirst")]
    pub native_first: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "pageToken")]
    pub page_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "pageSize")]
    pub page_size: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "syncCheck")]
    pub sync_check: Option<bool>,
}
impl Default for GetAccountBalanceRequest {
    fn default() -> Self {
        Self {
            blockchain: AnkrBlockchain::to_array_testnet(),
            wallet_address: String::new(),
            only_whitelisted: None,
            native_first: Some(true),
            page_token: None,
            page_size: None,
            sync_check: None,
        }
    }
}

// ====================== NFTs API ======================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Trait {
    pub trait_type: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AnkrNft {
    pub blockchain: AnkrBlockchain,
    pub name: String,
    #[serde(rename = "tokenId")]
    pub token_id: String,
    #[serde(rename = "tokenUrl")]
    pub token_url: String,
    #[serde(rename = "imageUrl")]
    pub image_url: String,
    #[serde(rename = "collectionName")]
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
        let token_id = str_to_u64(&self.token_id);
        let quantity = self.quantity.as_ref().map(|q| str_to_u64(q));
        let assets_type = self.contract_type.into_inter();

        Nft {
            chain_id: self.blockchain.to_chain_id(),
            address: Address::from_str(&self.contract_address).unwrap_or(Address::ZERO),
            name: self.name,
            symbol: self.symbol,
            token_id: Some(token_id),
            quantity,
            token_uri: Some(self.image_url),
            collection: Some(self.collection_name),
            assets_type: Some(assets_type)
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GetNFTsByOwnerRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blockchain: Option<Vec<AnkrBlockchain>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter: Option<Vec<HashMap<String, Vec<String>>>>, // 对应 TS 中的 { [key: string]: string[] }[]
    #[serde(rename = "walletAddress")]
    pub wallet_address: String,
    #[serde(skip_serializing_if = "Option::is_none", rename = "pageToken")]
    pub page_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "pageSize")]
    pub page_size: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "syncCheck")]
    pub sync_check: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GetNFTsByOwnerReply {
    pub owner: String,
    pub assets: Vec<AnkrNft>,
    #[serde(rename = "nextPageToken")]
    pub next_page_token: String,
    #[serde(skip_serializing_if = "Option::is_none", rename = "syncStatus")]
    pub sync_status: Option<SyncStatus>,
}

// ====================== Transactions API ======================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AnkrTransaction {
    // ====== 核心字段（必须保留）======
    #[serde(rename = "blockNumber")]
    pub block_number: String,
    pub from: String,
    #[serde(default)]
    pub to: Option<String>,
    pub hash: String,
    #[serde(rename = "gasPrice")]
    pub gas_price: Option<String>,
    pub value: String,
    #[serde(rename = "gasUsed")]
    pub gas_used: Option<String>,
    pub status: Option<String>,
    pub timestamp: Option<String>,
    pub blockchain: AnkrBlockchain,

    #[serde(default, skip)]
    pub v: Option<String>,
    #[serde(default, skip)]
    pub r: Option<String>,
    #[serde(default, skip)]
    pub s: Option<String>,
    #[serde(default, skip)]
    pub nonce: Option<String>,
    #[serde(default, skip)]
    pub gas: Option<String>,
    #[serde(default, skip)]
    pub input: Option<String>,
    #[serde(default, skip, rename = "transactionIndex")]
    pub transaction_index: Option<String>,
    #[serde(default, skip, rename = "blockHash")]
    pub block_hash: Option<String>,
    #[serde(default, skip)]
    pub r#type: Option<String>,
    #[serde(default, skip, rename = "contractAddress")]
    pub contract_address: Option<String>,
    #[serde(default, skip, rename = "cumulativeGasUsed")]
    pub cumulative_gas_used: Option<String>,
    #[serde(default, skip)]
    pub logs: Option<Vec<Log>>,
    #[serde(default, skip)]
    pub method: Option<Method>,
}

impl IntoInterTx for AnkrTransaction {
    fn into_inter(self) -> TransactionHistoryEntry {
        TransactionHistoryEntry {
            chain_id: self.blockchain.to_chain_id(),
            hash: self.hash.to_lowercase(),
            block_number: str_to_u64(&self.block_number),
            from: self.from.to_lowercase(),
            to: self.to.unwrap_or_default().to_lowercase(),
            value: self.value.and_then(|s| str_to_u256(s).ok()),
            gas_price: self.gas_price.and_then(|s| str_to_u256(s).ok()),
            gas_used: self.gas_used.and_then(|s| str_to_u256(s).ok()),
            status: self
                .status
                .filter(|s| s == "1")
                .map(|_| "success".to_string()),
            timestamp: self.timestamp.and_then(|s| str_to_u64(s).ok()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AnkrBlockIdentifier {
    Number(u64),
    Latest(String),
    Earliest(String),
}

impl AnkrBlockIdentifier {
    pub fn latest() -> Self {
        AnkrBlockIdentifier::Latest("latest".to_string())
    }

    pub fn earliest() -> Self {
        AnkrBlockIdentifier::Earliest("earliest".to_string())
    }
}

impl Default for AnkrBlockIdentifier {
    fn default() -> Self {
        AnkrBlockIdentifier::latest()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GetTransactionsByAddressRequest {
    #[serde(skip_serializing_if = "Option::is_none", rename = "fromBlock")]
    pub from_block: Option<AnkrBlockIdentifier>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "toBlock")]
    pub to_block: Option<AnkrBlockIdentifier>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "fromTimestamp")]
    pub from_timestamp: Option<AnkrBlockIdentifier>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "toTimestamp")]
    pub to_timestamp: Option<AnkrBlockIdentifier>,
    pub blockchain: Vec<AnkrBlockchain>, // 这里是必填数组（TS 中也是必填）
    #[serde(rename = "address")]
    pub address: Vec<String>, // 支持多个地址
    #[serde(skip_serializing_if = "Option::is_none", rename = "pageToken")]
    pub page_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "pageSize")]
    pub page_size: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "descOrder")]
    pub desc_order: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "includeLogs")]
    pub include_logs: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "syncCheck")]
    pub sync_check: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GetTransactionsByAddressReply {
    pub transactions: Vec<AnkrTransaction>,
    #[serde(rename = "nextPageToken")]
    pub next_page_token: String,
    #[serde(skip_serializing_if = "Option::is_none", rename = "syncStatus")]
    pub sync_status: Option<SyncStatus>,
}
