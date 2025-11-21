use alloy_primitives::{U256, U128};
use std::str::FromStr;
use crate::error::AppError;

/// 安全的 String → U256（最常用）
pub fn str_to_u256(s: impl AsRef<str>) -> Result<U256, AppError> {
    let s = s.as_ref().trim();
    if s.is_empty() || s == "0x" {
        return Ok(U256::ZERO);
    }
    // 支持 0x 前缀和纯数字
    U256::from_str_radix(s.strip_prefix("0x").unwrap_or(s), 16)
        .map_err(Into::into)
}

/// 安全的 String → U128（小金额用）
pub fn str_to_u128(s: impl AsRef<str>) -> Result<U256, AppError>  {
    let s = s.as_ref().trim();
    if s.is_empty() || s == "0x" {
        return Ok(0);
    }
    Ok(u128::from_str_radix(s.strip_prefix("0x").unwrap_or(s), 16)?)
}

/// U256 → String（十进制，适合存库或返回前端）
pub fn u256_to_string_dec(value: U256) -> String {
    value.to_string()
}

/// U256 → String（十六进制，前端常用）
pub fn u256_to_string_hex(value: U256) -> String {
    format!("0x{}", value.to_string())
}

/// U256 → f64（仅用于展示，永远不要用于计算！）
pub fn u256_to_f64_ether(value: U256) -> f64 {
    // 先转 u128 再除，防止溢出
    value.as_u128() as f64 / 1e18
}

pub fn u256_to_f64_gwei(value: U256) -> f64 {
    value.as_u128() as f64 / 1e9
}

pub fn u256_to_f64_wei(value: U256) -> f64 {
    value.as_u128() as f64
}

pub fn str_to_u64(input: &str) -> Result<u64, &'static str> {
    let s = input.trim();

    if s.is_empty() {
        return Err("input empty");
    }

    // hex: 0x1234_ab
    if let Some(hex) = s.strip_prefix("0x") {
        let cleaned: String = hex.chars().filter(|c| *c != '_').collect();
        return u64::from_str_radix(&cleaned, 16).map_err(|_| "invalid hex number");
    }

    // decimal: 1_000_000
    let cleaned: String = s.chars().filter(|c| *c != '_').collect();

    cleaned.parse::<u64>().map_err(|_| "invalid decimal number")
}

pub fn str_to_f64(input: &str) -> Result<f64, &'static str> {
    let s = input.trim();

    if s.is_empty() {
        return Err("input empty");
    }

    let cleaned: String = s.chars().filter(|c| *c != '_').collect();

    cleaned.parse::<f64>().map_err(|_| "invalid float number")
}


// 123456789 → "123_456_789"
// 1000 → "1_000"
pub fn u64_to_string(value: u64) -> String {
    let s = value.to_string();
    let mut out = String::new();
    let mut count = 0;

    for c in s.chars().rev() {
        if count == 3 {
            out.push('_');
            count = 0;
        }
        out.push(c);
        count += 1;
    }

    out.chars().rev().collect()
}



