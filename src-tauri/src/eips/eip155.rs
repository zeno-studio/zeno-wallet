use crate::core::state::{AppState, get_current_chain, get_persistent_config};
use crate::error::AppError;
use crate::rpc::https::EthRpcProvider;
use alloy_consensus::{Signed, TxLegacy};
use alloy_primitives::{Signature,Address,keccak256, Bytes, TxKind,B256, U256,ChainId,};
use serde::Deserialize;
use serde_json::Value;
use tauri::State;
use hex::{encode as hex_encode};

use alloy_rlp::{Encodable, Decodable, RlpEncodable, EMPTY_LIST_CODE};

#[derive(Debug, Clone, Deserialize)]
pub struct LegacyTxParams {
    pub to: Address,
    #[serde(deserialize_with = "crate::utils::serde::deserialize_u256")]
    pub value_wei: U256,
    pub input: Bytes,
}

pub async fn build_Legacy_tx(
    provider: EthRpcProvider,
    params: Value,
    state: State<'_, AppState>,
) -> Result<TxLegacy, AppError> {
    let chain_id = get_current_chain(state.clone())?;
    let persistent_config = get_persistent_config(state.clone())?;
    let from = persistent_config
        .current_account_address
        .ok_or(AppError::Parse("Current account address not set"))?;

    // 验证 chain_id

    let params: LegacyTxParams =
        serde_json::from_value(params).map_err(AppError::JsonParseError)?;

    // 3. 获取 nonce
    let nonce = provider
        .get_nonce(&from, "latest")
        .await
        .map_err(|e| AppError::HttpsRpcError(format!("Get nonce failed: {}", e)))?;

    // 4. 简化 fee 估算（使用固定值）
    let gas_price = provider
        .gas_price()
        .await
        .map_err(|e| AppError::HttpsRpcError(format!("Get gas price failed: {}", e)))?;

    // 6. 构造 Legacy 交易请求
    let tx = TxLegacy {
        chain_id: Some(chain_id),
        nonce,
        gas_price,
        gas_limit: 21000,
        to: TxKind::Call(params.to),
        value: params.value_wei,
        input: params.input,
    };
    Ok(tx)
}



pub async fn build_155_elp(tx: TxLegacy, sig: Signature) -> Result<String, AppError> {
    let signed = Signed::new_unhashed(tx, sig);
    let mut buf = Vec::with_capacity(signed.rlp_encoded_length());
    signed.rlp_encode(&mut buf);
    let result = format!("0x{}", hex_encode(buf));
    Ok(result)
}


pub fn legacy_eip155_signing_hash(
    tx: &LegacyTx,
) -> B256 {
    // EIP-155: keccak256(rlp([nonce, gasPrice, gasLimit, to, value, data, chainId, 0, 0]))
    let mut payload = Vec::with_capacity(512);
    // 9 个字段：6 个原始 + chainId + 0 + 0
    (
        tx.nonce,
        tx.gas_price,
        tx.gas_limit,
        tx.to,
        tx.value,
        &tx.input,
        tx.chain_id,
        0u8,
        0u8,
    ).encode(&mut payload);

    keccak256(&payload)
}


#[derive(RlpEncodable)]
struct LegacyTx {
    chain_id: ChainId,
    nonce: u64,
    gas_price: u128,
    gas_limit: u64,
    to:TxKind,     // None = contract creation
    value: U256,
    input: Bytes,
}

