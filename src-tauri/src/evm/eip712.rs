use crate::error::AppError;

use alloy_primitives::{B256, I256, U256, keccak256};
use hex::decode as hex_decode;
use serde_json::Value;
use std::collections::{BTreeSet, HashMap};
use std::str::FromStr;
use std::sync::{Mutex, OnceLock};

static TYPE_HASH_CACHE: OnceLock<Mutex<HashMap<String, B256>>> = OnceLock::new();

/// EIP-712 helper (完整实现，包含 type hash 缓存、多维数组、domain fallback 严格编码)
pub struct EIP712;

impl EIP712 {
    /// Compute the EIP-712 domain-separated message digest from JSON input.
    /// Keep signature stable for easy replacement, now returns AppError.
    pub fn hash_eip712_message(json: &str) -> Result<B256, AppError> {
        let value: Value = serde_json::from_str(json).map_err(|_| AppError::JsonParseError)?;

        let domain = value.get("domain").ok_or(AppError::MissingDomain)?;
        let types = value.get("types").ok_or(AppError::MissingTypes)?;
        let primary_type = value
            .get("primaryType")
            .and_then(|v| v.as_str())
            .ok_or(AppError::MissingPrimaryType)?;
        let message = value.get("message").ok_or(AppError::MissingMessage)?;

        let struct_hash = Self::struct_hash(types, primary_type, message)?;
        let domain_hash = if types.get("EIP712Domain").is_some() {
            Self::struct_hash(types, "EIP712Domain", domain)?
        } else {
            Self::hash_domain_fallback(domain)?
        };

        let mut digest_input = vec![0x19u8, 0x01u8];
        digest_input.extend_from_slice(domain_hash.as_slice());
        digest_input.extend_from_slice(struct_hash.as_slice());
        let digest = keccak256(&digest_input);
        Ok(digest)
    }

    /// Compute struct hash for a given type name and data object using EIP-712 rules.
    pub fn struct_hash(types: &Value, struct_name: &str, data: &Value) -> Result<B256, AppError> {
        let type_hash = Self::type_hash(types, struct_name)?;
        let mut encoded = type_hash.to_vec();

        let fields = types
            .get(struct_name)
            .and_then(|v| v.as_array())
            .ok_or(AppError::TypeFieldsNotArray)?;

        for field in fields {
            let name = field
                .get("name")
                .and_then(|v| v.as_str())
                .ok_or(AppError::FieldMissingName)?;
            let typ = field
                .get("type")
                .and_then(|v| v.as_str())
                .ok_or(AppError::FieldMissingType)?;
            let value = data.get(name).ok_or(AppError::MissingFieldValue)?;
            let encoded_value =
                Self::encode_value(types, typ, value).map_err(|_| AppError::JsonParseError)?;
            encoded.extend_from_slice(encoded_value.as_slice());
        }

        Ok(keccak256(&encoded))
    }

    /// Build the full type string and return its keccak256 hash per EIP-712.
    /// With a simple global cache (process-wide). Cache key: "{primary}:{type_string}".
    fn type_hash(types: &Value, primary_type: &str) -> Result<B256, AppError> {
        // Build the canonical type string first (we need it as cache key)
        let mut collected = BTreeSet::new();
        let mut path: Vec<String> = Vec::new();
        Self::collect_types(types, primary_type, &mut collected, &mut path)?;
        let primary_str = Self::build_type_str(types, primary_type)?;
        let mut type_str = primary_str.clone();
        for dep in &collected {
            if dep != primary_type {
                let dep_str = Self::build_type_str(types, dep)?;
                type_str.push_str(&dep_str);
            }
        }

        // Use cache
        let cache = TYPE_HASH_CACHE.get_or_init(|| Mutex::new(HashMap::new()));
        let key = format!("{}:{}", primary_type, type_str);
        if let Ok(mut m) = cache.lock() {
            if let Some(cached) = m.get(&key) {
                return Ok(*cached);
            }
            // compute and insert
            let hash = keccak256(type_str.as_bytes());
            m.insert(key.clone(), hash);
            return Ok(hash);
        } else {
            // Unable to lock cache -> fallback compute without cache
            let hash = keccak256(type_str.as_bytes());
            return Ok(hash);
        }
    }

    /// Construct a single type definition string, e.g., "Mail(Person from,Person to,string contents)".
    fn build_type_str(types: &Value, typ: &str) -> Result<String, AppError> {
        let fields = types
            .get(typ)
            .and_then(|v| v.as_array())
            .ok_or(AppError::TypeFieldsNotArray)?;

        let mut parts: Vec<String> = Vec::with_capacity(fields.len());
        for f in fields {
            let name = f
                .get("name")
                .and_then(|v| v.as_str())
                .ok_or(AppError::FieldMissingName)?;
            let field_typ = f
                .get("type")
                .and_then(|v| v.as_str())
                .ok_or(AppError::FieldMissingType)?;
            parts.push(format!("{} {}", field_typ, name));
        }
        Ok(format!("{}({})", typ, parts.join(",")))
    }

    /// Collect dependent custom types reachable from `typ`, detecting cycles.
    /// Support array element type recursion (multi-dimensional arrays included).
    fn collect_types(
        types: &Value,
        typ: &str,
        collected: &mut BTreeSet<String>,
        path: &mut Vec<String>,
    ) -> Result<(), AppError> {
        let typ_owned = typ.to_string();
        if path.contains(&typ_owned) {
            return Err(AppError::CycleDetected);
        }
        path.push(typ_owned.clone());

        if collected.insert(typ_owned.clone()) {
            let fields = types
                .get(&typ_owned)
                .and_then(|v| v.as_array())
                .ok_or(AppError::TypeFieldsNotArray)?;
            for field in fields {
                let field_typ = field
                    .get("type")
                    .and_then(|v| v.as_str())
                    .ok_or(AppError::FieldMissingType)?;
                let parsed = Self::parse_type(field_typ);
                if types.get(&parsed.base).is_some() {
                    Self::collect_types(types, &parsed.base, collected, path)?;
                }
            }
        }

        path.pop();
        Ok(())
    }

    /// Encode a value per EIP-712 type and return the 32-byte hashed/padded representation.
    /// Support multi-dimensional arrays.
    fn encode_value(types: &Value, typ: &str, value: &Value) -> Result<B256, AppError> {
        let parsed = Self::parse_type(typ);
        Self::encode_value_parsed(types, &parsed, value)
    }

    /// Recursive encoding based on ParsedType (for multi-dimensional arrays).
    fn encode_value_parsed(types: &Value, parsed: &ParsedType, value: &Value) -> Result<B256, AppError> {
        // If there are array dimensions -> current level is array
        if !parsed.array_dims.is_empty() {
            // value must be array
            let arr = value.as_array().ok_or(AppError::UnsupportedType)?;
            // check current (first) dimension length if fixed
            if let Some(Some(expected)) = parsed.array_dims.first()
                && arr.len() != *expected
            {
                return Err(AppError::UnsupportedType);
            }
            // For each element, encode with a parsed type that has one fewer dimension
            let mut concat: Vec<u8> = Vec::new();
            for elem in arr {
                let mut next = parsed.clone();
                next.array_dims.remove(0);
                // If array_dims becomes empty => element base type; else still array
                let e = Self::encode_value_parsed(types, &next, elem)?;
                concat.extend_from_slice(e.as_slice());
            }
            return Ok(keccak256(&concat));
        }

        // Not an array now
        if types.get(&parsed.base).is_some() {
            // custom struct
            return Self::struct_hash(types, &parsed.base, value);
        }

        // atomic
        Self::encode_atomic(&parsed.base, value)
    }

    /// Encode atomic (non-struct, non-array) base type.
    fn encode_atomic(base: &str, value: &Value) -> Result<B256, AppError> {
        match base {
            "address" => {
                let addr_str = Self::value_to_string(value)?;
                let addr_clean = addr_str.strip_prefix("0x").unwrap_or(&addr_str);
                let addr_bytes = hex_decode(addr_clean).map_err(|_| AppError::InvalidAddress)?;
                if addr_bytes.len() != 20 {
                    return Err(AppError::InvalidAddress);
                }
                let mut padded = [0u8; 32];
                padded[12..].copy_from_slice(&addr_bytes);
                Ok(B256::from(padded))
            }
            "bool" => {
                let b = match value {
                    Value::Bool(bb) => *bb,
                    Value::String(s) => s.to_lowercase() == "true",
                    _ => return Err(AppError::InvalidNumber),
                };
                let num = if b { U256::ONE } else { U256::ZERO };
                Ok(B256::from_slice(&num.to_be_bytes::<32>()))
            }
            t if t.starts_with("uint") => {
                let bits = Self::parse_bits(t, "uint")?;
                let num_str = Self::value_to_string(value)?;
                let num = U256::from_str(&num_str).map_err(|_| AppError::UnsupportedType)?;
                if bits < 256 && num.bit_len() > bits {
                    return Err(AppError::UnsupportedType);
                }
                Ok(B256::from_slice(&num.to_be_bytes::<32>()))
            }
            t if t.starts_with("int") => {
                let bits = Self::parse_bits(t, "int")?;
                let num_str = Self::value_to_string(value)?;
                let num = I256::from_str(&num_str).map_err(|_| AppError::UnsupportedType)?;
                if bits < 256 {
                    let abs = num.unsigned_abs();
                    let limit = if bits == 0 {
                        return Err(AppError::InvalidTypePrefix);
                    } else {
                        U256::ONE << (bits - 1)
                    };
                    let overflow = if num.is_negative() {
                        abs > limit
                    } else {
                        abs >= limit
                    };
                    if overflow {
                        return Err(AppError::ValueOverflow);
                    }
                }
                Ok(B256::from_slice(&num.to_be_bytes::<32>()))
            }
            t if t.starts_with("bytes") && t != "bytes" => {
                // bytesN: N is byte count (not bits), range 1..=32
                let bytes_len = Self::parse_bits(t, "bytes")?;
                let s = Self::value_to_string(value)?;
                let b = hex_decode(s.strip_prefix("0x").unwrap_or(&s))
                    .map_err(|_| AppError::InvalidBytesHex)?;
                if b.len() > bytes_len {
                    return Err(AppError::ValueOverflow);
                }
                let mut padded = [0u8; 32];
                padded[..b.len()].copy_from_slice(&b);
                Ok(B256::from(padded))
            }
            "bytes" => {
                let s = Self::value_to_string(value)?;
                let b = hex_decode(s.strip_prefix("0x").unwrap_or(&s))
                    .map_err(|_| AppError::InvalidBytesHex)?;
                Ok(keccak256(&b))
            }
            "string" => {
                let s = Self::value_to_string(value)?;
                Ok(keccak256(s.as_bytes()))
            }
            _ => Err(AppError::UnsupportedType),
        }
    }

    /// Parse a type string into its base type and vector of array dimensions (outermost first).
    /// Examples:
    /// - "uint256" -> base="uint256", array_dims=[]
    /// - "uint256[3][]" -> base="uint256", array_dims=[Some(3), None]
    fn parse_type(typ: &str) -> ParsedType {
        let mut base = String::new();
        let mut array_dims: Vec<Option<usize>> = Vec::new();

        let mut i = 0usize;
        let bytes = typ.as_bytes();
        while i < bytes.len() {
            if bytes[i] == b'[' {
                // start of array dims; capture base if not set
                if base.is_empty() {
                    base = typ[..i].to_string();
                }
                // find closing
                if let Some(j) = typ[i..].find(']') {
                    let inside = &typ[i + 1..i + j];
                    if inside.is_empty() {
                        array_dims.push(None);
                    } else {
                        array_dims.push(inside.parse::<usize>().ok());
                    }
                    i = i + j + 1;
                    continue;
                } else {
                    // malformed -> treat remainder as base
                    base = typ.to_string();
                    break;
                }
            } else {
                i += 1;
            }
        }
        if base.is_empty() {
            // no arrays found; entire typ is base
            base = typ.to_string();
        }

        ParsedType { base, array_dims }
    }

    /// Parse bit/size suffix and validate ranges.
    fn parse_bits(typ: &str, prefix: &str) -> Result<usize, AppError> {
        let suff = typ.strip_prefix(prefix).ok_or(AppError::InvalidTypePrefix)?;
        let bits = suff.parse::<usize>().map_err(|_| AppError::InvalidTypePrefix)?;
        match prefix {
            "bytes" => {
                if bits == 0 || bits > 32 {
                    return Err(AppError::InvalidTypePrefix);
                }
            }
            "uint" | "int" => {
                if bits == 0 || bits > 256 || (bits % 8 != 0) {
                    return Err(AppError::InvalidTypePrefix);
                }
            }
            _ => return Err(AppError::InvalidTypePrefix),
        }
        Ok(bits)
    }

    /// Convert JSON value to string for numeric/string/hex processing.
    fn value_to_string(value: &Value) -> Result<String, AppError> {
        match value {
            Value::String(s) => Ok(s.clone()),
            Value::Number(n) => Ok(n.to_string()),
            _ => Err(AppError::UnsupportedType),
        }
    }

    /// Domain fallback strict encoding when `types` does not define `EIP712Domain`.
    /// Fields: name(string), version(string), chainId(uint256), verifyingContract(address), salt(bytes32)
    /// Missing fields are zero-padded.
    fn hash_domain_fallback(domain: &Value) -> Result<B256, AppError> {
        // build type hash for canonical domain type
        let domain_type_str = "EIP712Domain(string name,string version,uint256 chainId,address verifyingContract,bytes32 salt)";
        let type_hash = keccak256(domain_type_str.as_bytes());

        // For each known field compute encoded 32 bytes (string/bytes -> keccak, uint -> numeric, address -> padded)
        let mut enc = type_hash.to_vec();

        // name
        if let Some(v) = domain.get("name") {
            match v {
                Value::String(s) => enc.extend_from_slice(keccak256(s.as_bytes()).as_slice()),
                _ => return Err(AppError::DomainFallbackInvalid),
            }
        } else {
            enc.extend_from_slice(&[0u8; 32]);
        }

        // version
        if let Some(v) = domain.get("version") {
            match v {
                Value::String(s) => enc.extend_from_slice(keccak256(s.as_bytes()).as_slice()),
                _ => return Err(AppError::DomainFallbackInvalid),
            }
        } else {
            enc.extend_from_slice(&[0u8; 32]);
        }

        // chainId (accept number or string numeric)
        if let Some(v) = domain.get("chainId") {
            let s = Self::value_to_string(v)?;
            let num = U256::from_str(&s).map_err(|_| AppError::DomainFallbackInvalid)?;
            enc.extend_from_slice(&num.to_be_bytes::<32>());
        } else {
            enc.extend_from_slice(&[0u8; 32]);
        }

        // verifyingContract (address)
        if let Some(v) = domain.get("verifyingContract") {
            let s = Self::value_to_string(v)?;
            let addr_clean = s.strip_prefix("0x").unwrap_or(&s);
            let addr_bytes = hex_decode(addr_clean).map_err(|_| AppError::DomainFallbackInvalid)?;
            if addr_bytes.len() != 20 {
                return Err(AppError::DomainFallbackInvalid);
            }
            let mut padded = [0u8; 32];
            padded[12..].copy_from_slice(&addr_bytes);
            enc.extend_from_slice(&padded);
        } else {
            enc.extend_from_slice(&[0u8; 32]);
        }

        // salt (bytes32 hex)
        if let Some(v) = domain.get("salt") {
            let s = Self::value_to_string(v)?;
            let b = hex_decode(s.strip_prefix("0x").unwrap_or(&s))
                .map_err(|_| AppError::DomainFallbackInvalid)?;
            if b.len() != 32 {
                return Err(AppError::DomainFallbackInvalid);
            }
            enc.extend_from_slice(&b);
        } else {
            enc.extend_from_slice(&[0u8; 32]);
        }

        Ok(keccak256(&enc))
    }
}

/// Parsed representation of a type with base name and ordered array dimensions (outermost first).
#[derive(Clone)]
struct ParsedType {
    base: String,
    array_dims: Vec<Option<usize>>,
}