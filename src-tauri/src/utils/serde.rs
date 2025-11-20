use alloy_primitives::{Address,U256, B256, Signature, keccak256};
use serde::{Deserialize, Deserializer, Serialize, Serializer, de};
use serde_json::Value;


// 自动把前端传来的 number | string 转成 U256（最稳）
pub fn deserialize_u256<'de, D>(deserializer: D) -> Result<U256, D::Error>
where
    D: Deserializer<'de>,
{
    let v = Value::deserialize(deserializer)?;
    match v {
        Value::String(s) => s.parse::<U256>().map_err(serde::de::Error::custom),
        Value::Number(n) if n.is_u64() => Ok(U256::from(n.as_u64().unwrap())),
        Value::Number(n) => Ok(U256::from(n.as_f64().unwrap() as u64)), // 理论上不会走到这
        _ => Err(serde::de::Error::custom("expected string or number")),
    }
}

// 发给前端时自动转 string
pub fn serialize_u256<S>(value: &U256, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&value.to_string())
}

pub fn serialize_f64<S>(value: &f64, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&value.to_string())
}

pub fn deserialize_f64_from_str<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    s.parse::<f64>().map_err(serde::de::Error::custom)
}

pub fn deserialize_option_f64_from_str<'de, D>(deserializer: D) -> Result<Option<f64>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt: Option<String> = Option::deserialize(deserializer)?;
    match opt {
        Some(s) => s.parse::<f64>().map(Some).map_err(serde::de::Error::custom),
        None => Ok(None),
    }
}


pub fn serialize_u128<S>(value: &u128, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&value.to_string())
}

pub fn deserialize_u128_from_str<'de, D>(deserializer: D) -> Result<u128, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    s.parse::<u128>().map_err(serde::de::Error::custom)
}

pub fn deserialize_option_u128_from_str<'de, D>(deserializer: D) -> Result<Option<u128>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: Option<String> = Option::deserialize(deserializer)?;
    match s {
        Some(str_val) => str_val.parse::<u128>().map(Some).map_err(serde::de::Error::custom),
        None => Ok(None),
    }
}

pub fn serialize_u64<S>(value: &u64, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&value.to_string())
}

pub fn deserialize_u64_from_str<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    if let Some(hex_val) = s.strip_prefix("0x") {
        u64::from_str_radix(hex_val, 16).map_err(|_| de::Error::custom(format!("Invalid hex string for u64: {s}")))
    } else {
        s.parse::<u64>().map_err(serde::de::Error::custom)
    }
}

pub fn deserialize_u64_from_str_or_int<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Value::deserialize(deserializer)?;
    match value {
        Value::Number(num) => num.as_u64().ok_or_else(|| de::Error::custom(format!("Invalid number for u64: {num}"))),
        Value::String(s) => {
            if let Some(hex_val) = s.strip_prefix("0x") {
                u64::from_str_radix(hex_val, 16).map_err(|_| de::Error::custom(format!("Invalid hex string for u64: {s}")))
            } else {
                s.parse::<u64>().map_err(|_| de::Error::custom(format!("Invalid decimal string for u64: {s}")))
            }
        }
        _ => Err(de::Error::custom("u64 must be a number or a string")),
    }
}

pub fn deserialize_option_u64_from_str<'de, D>(deserializer: D) -> Result<Option<u64>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: Option<String> = Option::deserialize(deserializer)?;
    match s {
        Some(str_val) => str_val.parse::<u64>().map(Some).map_err(serde::de::Error::custom),
        None => Ok(None),
    }
}