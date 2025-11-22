use core::fmt;
use tauri::ipc::InvokeError;
use serde::{Serialize,Deserialize};
use reqwest::Error;


use alloy_primitives::{ParseSignedError, ruint};
use helios::client::ClientError;

#[derive(Debug, Serialize,Deserialize)]
pub enum AppError {
    // Standard errors
    Io(std::io::Error),
    Parse(&'static str),
    JsonParseError(serde_json::Error),
    NumberParseError(),
    HexDecodeError(hex::FromHexError),
    InvalidAddressLength(u64),
    InvalidNumber(std::num::ParseIntError),
    InvalidBytesHex(hex::FromHexError),
    InvalidBooleanParse(serde_json::Value),
    
    // Reqwest errors
    ReqwestClientBuildError(reqwest::Error),
    ReqwestClientConnectionError(reqwest::Error),
    HttpsRpcError(u64, String),
    
    //eip712
    Eip712MissingDomain,
    Eip712MissingTypes,
    Eip712MissingPrimaryType,
    Eip712MissingMessage,
    Eip712TypeFieldsNotArray,
    Eip712FieldMissingName,
    Eip712FieldMissingType,
    Eip712MissingFieldValue,
    Eip712CycleDetected,
    Eip712UnsupportedType,
    Eip712InvalidTypePrefix,
    Eip712ValueOverflow,
    Eip712DomainFallbackInvalid,
    
    // Database errors
    DbNotInitialized,
    DbColumnFamilyNotFound,
    DbSerializationError(String),
    DbDeserializationError(String),
    DbWriteError(String),
    DbReadError(String),
    DbKeyNotFound,
    DbAccountNotFound(u64),
    DbVaultNotFound(String),
    
    // Wallet Core errors
    WalletCoreError(String),
    // state errors
    AlreadyInitialized,
    InvalidPassword,
    
    // Helios errors
    HeliosClientError(String),
    HeliosInvalidUtf8,
    HeliosInvalidJson,
    HeliosInvalidAddress,
    HeliosInvalidBlockTag,
    HeliosInvalidCallRequest,
    HeliosInvalidTransaction,
    HeliosInvalidStoragePosition,
    
    // JSON RPC errors
    JsonRpcInvalidResponse,
    JsonRpcMissingResult,
    JsonRpcInvalidId,
    GatewayHostUnhealthy,
    
    // Parameter errors
    MissingParam(usize),
    InvalidParam(usize),
    
    // Unsupported method error
    UnsupportedMethod(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Io(e) => write!(f, "IO error: {}", e),
            AppError::Parse(s) => write!(f, "Parse error: {}", s),
            AppError::JsonParseError(e) => write!(f, "Json parse error:{}", e),
            AppError::NumberParseError() => write!(f, "Number parse error"),
            AppError::HexDecodeError(e) => write!(f, "Hex decode error: {}", e),
            AppError::InvalidBooleanParse(e) => write!(f, "Invalid boolean parse: {}", e),
            AppError::InvalidAddressLength(e) => write!(f, "Invalid address: {}", e),
            AppError::InvalidNumber(e) => write!(f, "Invalid number: {}", e),
            AppError::InvalidBytesHex(e) => write!(f, "Invalid bytes hex: {}", e),
            AppError::ReqwestClientBuildError(e) => write!(f, "Failed to create reqwest client: {}", e),
            AppError::ReqwestClientConnectionError(e) => write!(f, "Reqwest client error: {}", e),
            AppError::HttpsRpcError(code, message) => write!(f, "HTTPS RPC error {}: {}", code, message),
            
            //eip712
            AppError::Eip712MissingDomain => write!(f, "EIP712 domain missing"),
            AppError::Eip712MissingTypes => write!(f, "EIP712 types missing"),
            AppError::Eip712MissingPrimaryType => write!(f, "primaryType missing or not a string"),
            AppError::Eip712MissingMessage => write!(f, "message object missing"),
            AppError::Eip712TypeFieldsNotArray => write!(f, "type definition must be an array"),
            AppError::Eip712FieldMissingName => write!(f, "field missing \"name\""),
            AppError::Eip712FieldMissingType => write!(f, "field missing \"type\""),
            AppError::Eip712MissingFieldValue => write!(f, "field value missing in message"),
            AppError::Eip712CycleDetected => write!(f, "type definition cycle detected"),
            AppError::Eip712UnsupportedType => write!(f, "unsupported EIP712 type"),
            AppError::Eip712InvalidTypePrefix => write!(f, "invalid type prefix/suffix"),
            AppError::Eip712ValueOverflow => write!(f, "numeric value overflow"),
            AppError::Eip712DomainFallbackInvalid => write!(f, "invalid domain fallback field"),
            // Database errors
            AppError::DbNotInitialized => write!(f, "Database not initialized"),
            AppError::DbColumnFamilyNotFound => write!(f, "Database column family not found"),
            AppError::DbSerializationError(e) => write!(f, "Database serialization error: {}", e),
            AppError::DbDeserializationError(e) => {
                write!(f, "Database deserialization error: {}", e)
            }
            AppError::DbWriteError(e) => write!(f, "Database write error: {}", e),
            AppError::DbReadError(e) => write!(f, "Database read error: {}", e),
            AppError::DbKeyNotFound => write!(f, "Database key not found"),
            AppError::DbAccountNotFound(index) => {
                write!(f, "Database account not found: {}", index)
            }
            AppError::DbVaultNotFound(key) => write!(f, "Database vault not found: {}", key),

            // Wallet Core errors
            AppError::WalletCoreError(e) => write!(f, "Wallet core error: {}", e),
            // state errors
            AppError::AlreadyInitialized => write!(f, "Already initialized"),
            AppError::InvalidPassword => write!(f, "Invalid password"),

            // Helios errors
            AppError::HeliosClientError(e) => write!(f, "Helios client error: {}", e),
            AppError::HeliosInvalidUtf8 => write!(f, "Invalid UTF-8 in request body"),
            AppError::HeliosInvalidJson => write!(f, "Invalid JSON in request"),
            AppError::HeliosInvalidAddress => write!(f, "Invalid address format"),
            AppError::HeliosInvalidBlockTag => write!(f, "Invalid block tag"),
            AppError::HeliosInvalidCallRequest => write!(f, "Invalid call request"),
            AppError::HeliosInvalidTransaction => write!(f, "Invalid transaction format"),
            AppError::HeliosInvalidStoragePosition => write!(f, "Invalid storage position"),

            // JSON RPC errors
            AppError::JsonRpcInvalidResponse => write!(f, "Invalid JSON RPC response"),
            AppError::JsonRpcMissingResult => write!(f, "Missing result in JSON RPC response"),
            AppError::JsonRpcInvalidId => write!(f, "Invalid ID in JSON RPC response"),
            AppError::GatewayHostUnhealthy => write!(f, "Gateway host unhealthy"),
            
            // Parameter errors
            AppError::MissingParam(index) => write!(f, "Missing parameter at index {}", index),
            AppError::InvalidParam(index) => write!(f, "Invalid parameter at index {}", index),
            
            // Unsupported method error
            AppError::UnsupportedMethod(method) => write!(f, "Unsupported method: {}", method),
        }
    }
}

impl std::error::Error for AppError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            AppError::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<serde_json::Error> for AppError {
    fn from(error: serde_json::Error) -> Self {
        AppError::JsonParseError(error)
    }
}

impl From<ruint::ParseError> for AppError {
    fn from(_error: ruint::ParseError) -> Self {
        // Create a ParseIntError by parsing an invalid string
        let err = "invalid".parse::<u32>().unwrap_err();
        AppError::InvalidNumber(err)
    }
}

impl From<ParseSignedError> for AppError {
    fn from(_error: ParseSignedError) -> Self {
        // Create a ParseIntError by parsing an invalid string
        let err = "invalid".parse::<u32>().unwrap_err();
        AppError::InvalidNumber(err)
    }
}

impl From<hex::FromHexError> for AppError {
    fn from(error: hex::FromHexError) -> Self {
        AppError::InvalidBytesHex(error)
    }
}

impl From<std::num::ParseIntError> for AppError {
    fn from(error: std::num::ParseIntError) -> Self {
        AppError::InvalidNumber(error)
    }
}

impl From<ClientError> for AppError {
    fn from(error: ClientError) -> Self {
        AppError::HeliosClientError(error.to_string())
    }
}

// 为数据库模块提供From trait实现，方便错误转换
impl From<AppError> for String {
    fn from(error: AppError) -> String {
        error.to_string()
    }
}

// 为 Tauri 命令提供错误转换
impl From<AppError> for InvokeError {
    fn from(error: AppError) -> Self {
        InvokeError::from(error.to_string())
    }
}