use crate::core::state::{AppState, get_current_chain, get_ui_config};
use crate::error::AppError;
use crate::rpc::https::EthRpcProvider;
use alloy_consensus::TxLegacy;
use alloy_primitives::{Address, Bytes, TxKind, U256};
use serde::Deserialize;
use serde_json::Value;
use tauri::State;

#[derive(Debug, Clone, Deserialize)]
pub struct LegacyTxParams {
    pub to: Address,
    pub value_wei: U256,
    pub input: Bytes,
}

pub async fn build_Legacy_tx(
    provider: EthRpcProvider,
    params: Value,
    state: State<'_, AppState>,
) -> Result<TxLegacy, AppError> {
    let chain_id = get_current_chain(state.clone())?;
    let ui_config = get_ui_config(state.clone())?;
    let from = ui_config
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