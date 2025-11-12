use core::fmt;
use tauri::ipc::InvokeError;

#[derive(Debug)]
pub enum AppError {
    Io(std::io::Error),
    Parse(&'static str),
    JsonParseError(serde_json::Error),

    //eip712
    MissingDomain,
    MissingTypes,
    MissingPrimaryType,
    MissingMessage,
    TypeFieldsNotArray,
    FieldMissingName,
    FieldMissingType,
    MissingFieldValue,
    CycleDetected,
    UnsupportedType,
    InvalidAddress,
    InvalidNumber,
    InvalidBytesHex,
    InvalidTypePrefix,
    ValueOverflow,
    DomainFallbackInvalid,
    
    // Database errors
    DbNotInitialized,
    DbColumnFamilyNotFound,
    DbSerializationError(String),
    DbDeserializationError(String),
    DbWriteError(String),
    DbReadError(String),
    DbKeyNotFound,
    
    // Wallet Core errors
    WalletCoreError(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Io(e) => write!(f, "IO error: {}", e),
            AppError::Parse(s) => write!(f, "Parse error: {}", s),
            AppError::JsonParseError(e) => write!(f,"Json parse error:{}",e),
            //eip712
            AppError::MissingDomain => write!(f, "EIP712 domain missing"),
            AppError::MissingTypes => write!(f, "EIP712 types missing"),
            AppError::MissingPrimaryType => write!(f, "primaryType missing or not a string"),
            AppError::MissingMessage => write!(f, "message object missing"),
            AppError::TypeFieldsNotArray => write!(f, "type definition must be an array"),
            AppError::FieldMissingName => write!(f, "field missing \"name\""),
            AppError::FieldMissingType => write!(f, "field missing \"type\""),
            AppError::MissingFieldValue => write!(f, "field value missing in message"),
            AppError::CycleDetected => write!(f, "type definition cycle detected"),
            AppError::UnsupportedType => write!(f, "unsupported EIP712 type"),
            AppError::InvalidAddress => write!(f, "invalid address (must be 20 bytes)"),
            AppError::InvalidNumber => write!(f, "invalid numeric value"),
            AppError::InvalidBytesHex => write!(f, "invalid hex string for bytes"),
            AppError::InvalidTypePrefix => write!(f, "invalid type prefix/suffix"),
            AppError::ValueOverflow => write!(f, "numeric value overflow"),
            AppError::DomainFallbackInvalid => write!(f, "invalid domain fallback field"),
            // Database errors
            AppError::DbNotInitialized => write!(f, "Database not initialized"),
            AppError::DbColumnFamilyNotFound => write!(f, "Database column family not found"),
            AppError::DbSerializationError(e) => write!(f, "Database serialization error: {}", e),
            AppError::DbDeserializationError(e) => write!(f, "Database deserialization error: {}", e),
            AppError::DbWriteError(e) => write!(f, "Database write error: {}", e),
            AppError::DbReadError(e) => write!(f, "Database read error: {}", e),
            AppError::DbKeyNotFound => write!(f, "Database key not found"),
            // Wallet Core errors
            AppError::WalletCoreError(e) => write!(f, "Wallet core error: {}", e),
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

/// 数据库错误处理工具函数
pub struct DbErrorHandler;

impl DbErrorHandler {
    /// 处理Tauri命令中的数据库错误
    /// 将AppError转换为String以保持与前端的兼容性
    pub fn handle_command_error(error: AppError) -> String {
        error.to_string()
    }
    
    /// 记录数据库错误但不中断程序执行
    pub fn log_error(error: AppError) {
        eprintln!("Database error: {}", error);
    }
}

/// 数据库操作结果类型别名
pub type DbResult<T> = std::result::Result<T, AppError>;