use alloy_primitives::{keccak256, B256};
use crate::error::AppError;


pub fn hash_eip191_message(
    message: &str,
) -> Result<B256, AppError> {
    let prefix = format!("\x19Ethereum Signed Message:\n{}", message.len());
    let digest = keccak256([prefix.as_bytes(), message.as_bytes()].concat());
    Ok(digest)
}




