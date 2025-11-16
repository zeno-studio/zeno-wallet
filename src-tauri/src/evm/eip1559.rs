use crate::core::state::{AppState, get_current_chain, get_ui_config};
use crate::error::AppError;
use crate::rpc::https::EthRpcProvider;
use alloy_consensus::TxEip1559;
use alloy_eips::eip1559::BaseFeeParams;
use alloy_eips::eip2930::AccessList;
use alloy_primitives::{Address, Bytes, TxKind, U256};
use serde::Deserialize;
use serde_json::Value;
use tauri::State;

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
    let ui_config = get_ui_config(state.clone())?;
    let from = ui_config
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
