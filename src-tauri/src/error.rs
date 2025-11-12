use core::fmt;

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