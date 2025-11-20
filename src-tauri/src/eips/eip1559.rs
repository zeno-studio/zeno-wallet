use crate::core::state::{AppState, get_current_chain, get_persistent_config};
use crate::error::AppError;
use crate::rpc::https::EthRpcProvider;
use alloy_consensus::{Signed, TxEip1559};
use alloy_eips::eip1559::BaseFeeParams;
use alloy_eips::eip2930::AccessList;
use alloy_primitives::{Address, Bytes,B256, Signature, ChainId,TxKind, U256, keccak256};
use serde::Deserialize;
use serde_json::Value;
use tauri::State;
use hex::{encode as hex_encode};
use alloy_rlp::{Encodable, Decodable, RlpEncodable, EMPTY_LIST_CODE};

#[derive(Debug, Clone, Deserialize)]
pub struct Eip1559TxParams {
    pub to: Address,
    pub value_wei: U256,
    pub input: Bytes,
    pub access_list: AccessList,
}

pub async fn build_eip1559_tx(
    provider: EthRpcProvider,
    params: Value,
    state: State<'_, AppState>,
) -> Result<TxEip1559, AppError> {
    let chain_id = get_current_chain(state.clone())?;
    let persistent_config = get_persistent_config(state.clone())?;
    let from = persistent_config
        .current_account_address
        .ok_or(AppError::Parse("Current account address not set"))?;

    // 验证 chain_id

    let params: Eip1559TxParams =
        serde_json::from_value(params).map_err(AppError::JsonParseError)?;

    // 3. 获取 nonce
    let nonce = provider
        .get_nonce(&from, "latest")
        .await
        .map_err(|e| AppError::HttpsRpcError(format!("Get nonce failed: {}", e)))?;

    // 4. 简化 fee 估算（使用固定值）
    let base_fee_per_gas = 10_000_000_000u128; // 10 Gwei as example
    let max_priority_fee_per_gas = 2_000_000_000u128; // 2 Gwei
    let max_fee_per_gas = base_fee_per_gas + max_priority_fee_per_gas;

    // 6. 构造 EIP-1559 交易请求
    let tx = TxEip1559 {
        chain_id,
        nonce: nonce as u64,
        gas_limit: 21000,
        max_fee_per_gas: max_fee_per_gas as u128,
        max_priority_fee_per_gas: max_priority_fee_per_gas as u128,
        to: TxKind::Call(params.to),
        value: params.value_wei,
        input: params.input,
        access_list: params.access_list,
    };
    Ok(tx)
}

pub fn eip1559_signing_hash(tx: &TxEip1559) -> B256 {
    // 交易类型前缀 0x02 + rlp([chain_id, nonce, max_priority_fee, max_fee, gas_limit, destination, value, data, access_list])
    let mut payload = Vec::with_capacity(512);
    payload.push(0x02);                     // TransactionType

    let envelope = Eip1559Envelope {
        chain_id: tx.chain_id,
        nonce: tx.nonce,
        max_priority_fee_per_gas: tx.max_priority_fee_per_gas,
        max_fee_per_gas: tx.max_fee_per_gas,
        gas_limit: tx.gas_limit,
        destination: tx.to,
        value: tx.value,
        data: tx.input.clone(),
        access_list: tx.access_list.clone(),
    };
    envelope.encode(&mut payload);

    keccak256(&payload)
}

#[derive(RlpEncodable)]
struct Eip1559Envelope {
    chain_id: ChainId,
    nonce: u64,
    max_priority_fee_per_gas: u128,
    max_fee_per_gas: u128,
    gas_limit: u64,
    destination: TxKind,     // Call(addr) 或 Create
    value: U256,
    data: Bytes,
    access_list: AccessList,
}


pub async fn build_1559_elp(
   tx:TxEip1559,
   sig:Signature,
) -> Result<String, AppError> {
     let signed = Signed::new_unhashed(tx, sig);
    let mut buf = Vec::with_capacity(signed.rlp_encoded_length());
    signed.rlp_encode(&mut buf);
    let result = format!("0x{}", hex_encode(buf));
    Ok(result)
}