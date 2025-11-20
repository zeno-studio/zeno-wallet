use crate::core::state::{AppState, get_current_chain, get_persistent_config};
use crate::error::AppError;
use crate::rpc::https::EthRpcProvider;
use z_wallet_core::WalletCore;
use alloy_consensus::{Signed, TxEip7702};
use alloy_eips::eip7702::{Authorization, SignedAuthorization};
use alloy_eips::eip2930::AccessList;
use alloy_primitives::{Address, Bytes,B256,ChainId, keccak256,TxKind, U256, Signature};
use serde::Deserialize;
use serde_json::Value;
use tauri::State;
use hex::{encode as hex_encode};
use alloy_rlp::{Encodable, Decodable, RlpEncodable, EMPTY_LIST_CODE};
use alloy_sol_types::Eip712Domain; 

#[derive(Debug, Clone, Deserialize)]
pub struct Eip7702TxParams {
    pub to: Address,
    pub value_wei: U256,
    pub input: Bytes,
    pub access_list: AccessList,
    pub authorization_list_unsign: Vec<Authorization>,
}

pub async fn build_eip7702_tx_no_signedAuthorization(
    provider: EthRpcProvider,
    params: Value,
    state: State<'_, AppState>,
    wallet: WalletCore
) -> Result<TxEip7702, AppError> {
    let chain_id = get_current_chain(state.clone())?;
    let persistent_config = get_persistent_config(state.clone())?;
    let from = persistent_config
        .current_account_address
        .ok_or(AppError::Parse("Current account address not set"))?;

    // 验证 chain_id

    let params: Eip7702TxParams =
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

    // 处理授权列表
    let mut signed_authorization_list = Vec::new();
    for authorization in &params.authorization_list_unsign {
        // 创建一个空的签名（实际应用中需要正确签名）
        let sig = wallet.sign_tx(&authorization)?;
        signed_authorization_list.push(authorization.clone().into_signed(sig));
    }

    // 6. 构造 EIP-7702 交易请求
    let tx = TxEip7702 {
        chain_id,
        nonce: nonce as u64,
        gas_limit: 21000,
        max_fee_per_gas: max_fee_per_gas as u128,
        max_priority_fee_per_gas: max_priority_fee_per_gas as u128,
        to: params.to,
        value: params.value_wei,
        input: params.input,
        access_list: params.access_list.clone(),
        authorization_list: SignedAuthorization::new(),
    };
    Ok(tx)
}

pub async fn build_7702_elp(
   tx:TxEip7702,
   sig:Signature,
) -> Result<String, AppError> {
     let signed = Signed::new_unhashed(tx, sig);
    let mut buf = Vec::with_capacity(signed.rlp_encoded_length());
    signed.rlp_encode(&mut buf);
    let result = format!("0x{}", hex_encode(buf));
    Ok(result)
}




// ======================== 3. EIP-7702 ========================
pub fn eip7702_signing_hash(tx: &TxEip7702) -> B256 {
    // 交易类型前缀 0x04 + rlp([chain_id, nonce, max_prio, max_fee, gas_limit, to, value, data, access_list, authorization_list])
    let mut payload = Vec::with_capacity(1024);
    payload.push(0x04);                     // TransactionType

    let envelope = Eip7702Envelope {
        chain_id: tx.chain_id,
        nonce: tx.nonce,
        max_priority_fee_per_gas: tx.max_priority_fee_per_gas,
        max_fee_per_gas: tx.max_fee_per_gas,
        gas_limit: tx.gas_limit,
        destination: tx.to,
        value: tx.value,
        data: tx.input,
        access_list: tx.access_list,
        authorization_list: tx.authorization_list,
    };
    envelope.encode(&mut payload);

    keccak256(&payload)
}

#[derive(RlpEncodable)]
struct Eip7702Envelope {
    chain_id: ChainId,
    nonce: u64,
    max_priority_fee_per_gas: u128,
    max_fee_per_gas: u128,
    gas_limit: u64,
    destination: Address,     // Call(addr) 或 Create
    value: U256,
    data: Bytes,
    access_list: AccessList,
    authorization_list: Vec<SignedAuthorization>,
}

// ======================== Authorization 的 EIP-712 hash（必须先算这个） ========================
pub fn authorization_signing_hash(
    auth: &Authorization,
    domain: &Eip712Domain,   // 通常是 chain_id + verifyingContract = sender address
) -> B256 {
    // 参考 EIP-7702 正式规范
    let encoded = auth.encode_eip712(domain).expect("valid auth");
    keccak256(encoded)
}

// 简化版 Authorization 结构（实际跟 alloy-network 的完全一致）
#[derive(RlpEncodable, Clone)]
pub struct Authorization {
    pub chain_id: ChainId,
    pub address: Address,       // 要临时设置 code 的合约地址
    pub nonce: u64,
    #[rlp(default)]
    pub signature: Vec<u8>,     // 65 bytes，签完后填进来
}

// alloy-sol-types 风格的 EIP-712 encode（你也可以直接手写）
impl Authorization {
    pub fn encode_eip712(&self, domain: &Eip712Domain) -> Result<Vec<u8>, alloy_sol_types::Error> {
        use alloy_sol_types::{SolStruct, eip712::EIP712Domain};
        // 这里用 alloy 的宏自动实现（推荐）
        // 或者手写：keccak256("\x19\x01" || domain_hash || struct_hash)
        Ok(alloy_sol_types::Eip712::encode_eip712(self, domain)?)
    }
}