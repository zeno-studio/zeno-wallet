use serde::{Deserialize, Serialize};
use serde_json::json;
use reqwest::Client;
use crate::error::AppError;
use crate::rpc::ankr::models::*;

use crate::constants::{ANKR_CHAINS, ANKR_TESTCHAINS};



use std::collections::HashMap;



// ====================== 共用类型 ======================


// ====================== Transactions API ======================


pub async fn get_token_balances_by_ankr(
    client: &Client,
    address: &str,
    gateway_url: &str,
    allow_testnets: bool,
) -> anyhow::Result<GetAccountBalanceReply> {
    let chains = if allow_testnets {
        AnkrBlockchain::to_array_testnet()
    } else {
        AnkrBlockchain::to_array()
    }; 
    let params = GetAccountBalanceRequest {
        blockchain: chains,
        wallet_address: address.to_string(),
        native_first: Some(true),
        ..Default::default()
    };

    let body = json!({
        "id": 1,
        "jsonrpc": "2.0",
        "method": "ankr_getAccountBalance",
        "params": {
            "blockchain": chains,
            "onlyWhitelisted": true,
            "walletAddress": address,
            "nativeFirst": true
        }
    });

    let res = client
        .post(gateway_url)
        .json(&body)
        .send()
        .await?
        .error_for_status()?   
        .json::<serde_json::Value>()
        .await?;

    let parsed = serde_json::from_value(res["result"].clone())?;
    Ok(parsed)
}


pub async fn get_nft_balances_by_ankr(
    client: &Client,
    api_key: &str,

    
    chains: &[String],
    address: &str,
    page_token: Option<String>,
) -> anyhow::Result<GetNFTsByOwnerReply> {
    let body = json!({
        "id": 1,
        "jsonrpc": "2.0",
        "method": "ankr_getNFTsByOwner",
        "params": {
            "blockchain": chains,
            "walletAddress": address,
            "pageSize": 50,
            "pageToken": page_token
        }
    });

    let res = client
        .post(format!("https://rpc.ankr.com/multichain/{}", api_key))
        .json(&body)
        .send()
        .await?
        .error_for_status()?
        .json::<serde_json::Value>()
        .await?;

    let parsed = serde_json::from_value(res["result"].clone())?;
    Ok(parsed)
}


pub async fn get_activity_by_ankr(
    client: &Client,
    api_key: &str,
    chains: &[String],
    address: &str,
    days: u64,
    page_token: Option<String>,
) -> anyhow::Result<GetTransactionsByAddressReply> {
    let now = chrono::Utc::now().timestamp();
    let from_timestamp = now - (days as i64) * 24 * 60 * 60;

    let body = json!({
        "id": 1,
        "jsonrpc": "2.0",
        "method": "ankr_getTransactionsByAddress",
        "params": {
            "blockchain": chains,
            "address": address,
            "fromTimestamp": from_timestamp,
            "descOrder": true,
            "pageToken": page_token,
            "pageSize": 50
        }
    });

    let res = client
        .post(format!("https://rpc.ankr.com/multichain/{}", api_key))
        .json(&body)
        .send()
        .await?
        .error_for_status()?
        .json::<serde_json::Value>()
        .await?;

    let parsed = serde_json::from_value(res["result"].clone())?;
    Ok(parsed)
}


