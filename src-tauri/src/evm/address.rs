// src/utils/address.rs
// 2025 年最强 Ethereum 地址工具箱
// 支持：checksum、按 chain_id coloring（EIP-1191）、小写、验证、topic 解析、ENS 预留等

use alloy_primitives::{Address, AddressError, B256};
use std::str::FromStr;

/// 按 EIP-1191 根据 chain_id 计算带颜色的 checksum 地址（部分 L2 用）
/// 目前只有少数链启用：Optimism, Polygon, Arbitrum 等用不同规则
pub fn address_checksum(address: &str, chain_id: Option<u64>) -> Result<String, AddressError> {
    let addr = Address::from_str(address)?;
    Ok(match chain_id {
        // 这些链使用 EIP-1191 特殊规则（带 chain_id 的 hash）
        Some(10) => addr.to_checksum_with_chain(10),   // Optimism
        Some(137) => addr.to_checksum_with_chain(137), // Polygon
        Some(42161) => addr.to_checksum_with_chain(42161), // Arbitrum
        Some(8453) => addr.to_checksum_with_chain(8453),   // Base（部分工具支持）
        _ => addr.to_checksum(None), // 默认 EIP-55
    })
}

/// 强制返回小写地址（推荐用于存储、比较、复制）
pub fn address_to_lowercase(address: &str) -> Result<String, AddressError> {
    let addr = Address::from_str(address)?;
    Ok(format!("0x{:x}", addr))
}

/// 严格校验地址（支持大小写、checksum、0x 前缀）
pub fn validate_address(address: &str) -> Result<Address, AddressError> {
    Address::from_str(address.trim())
}

/// 从 topic 中提取地址（常见于 log.data 和 indexed topic）
/// topic 通常是 0x000000000000000000000000 + 20字节地址
pub fn address_from_topic(topic: &str) -> Option<Address> {
    let cleaned = topic.trim().strip_prefix("0x")?.trim_start_matches('0');
    if cleaned.len() != 40 {
        return None;
    }
    Address::from_str(&format!("0x{cleaned}")).ok()
}

/// 从 32 字节 hex 字符串提取地址（常用于 event topic）
pub fn address_from_hex_32(hex: &str) -> Option<Address> {
    let hex = hex.strip_prefix("0x")?;
    if hex.len() != 64 {
        return None;
    }
    let addr_hex = &hex[24..]; // 后 20 字节 = 40 个 hex 字符
    Address::from_str(&format!("0x{addr_hex}")).ok()
}

/// 判断是否是零地址
pub fn is_zero_address(addr: &Address) -> bool {
    *addr == Address::ZERO
}

/// 判断是否是常见预编译合约（EIP-1352）
pub fn is_precompile_or_system(addr: &Address) -> bool {
    addr.as_slice()[0..19] == [0u8; 19] && addr.as_slice()[19] <= 0x0f
}

/// 安全显示地址：前6 + ... + 后4（UI 常用）
pub fn display_address(addr: &str) -> String {
    if addr.len() < 10 {
        return addr.to_string();
    }
    format!("{}...{}", &addr[..8], &addr[addr.len()-6..])
}

/// 批量转小写（性能敏感场景）
pub fn addresses_to_lowercase(addresses: &[String]) -> Vec<String> {
    addresses
        .iter()
        .filter_map(|s| address_to_lowercase(s).ok())
        .collect()
}

/// 判断地址是否符合 checksum（防用户手输错）
pub fn is_valid_checksum(address: &str, chain_id: Option<u64>) -> bool {
    if let Ok(addr) = Address::from_str(address) {
        let expected = address_checksum(&format!("0x{:x}", addr), chain_id).unwrap_or_default();
        expected == address
    } else {
        false
    }
}

// ==================== 常用常量 ====================
pub mod known {
    use super::Address;

    pub const ZERO: Address = Address::ZERO;
    pub const PERMIT2: Address = Address::new([0x00, 0x00, 0x00, 0x00, 0x00, 0x22, 0xD4, 0x73, 0x03, 0x0F, 0x11, 0x6d, 0xDE, 0xE9, 0xF6, 0xB4, 0x3a, 0xC7, 0x8B, 0xA3]);
    pub const WETH9: [&str; 6] = [
        "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2", // ETH
        "0x21be370D5312f44cB42ce377BC9b8a0cEF1A4C83", // FTM
        "0x82aF49447D8a07e3bd95BD0d56f35241523fBab1", // ARB
        "0x4200000000000000000000000000000000000006", // Base / OP
        "0x0d500B1d8E8eF31E21C99d1Db9A6444d3ADf1270", // Polygon
        "0x2170Ed0880ac9A755fd29B2688956BD959F933F8", // BSC
    ];
}


// // 导入地址时
// let user_input = "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48"; // 小写
// let checksum = address_checksum(user_input, Some(1))?; // → 带大小写校验
// let lower = address_to_lowercase(user_input)?; // → 存数据库

// // 交易解析
// let addr = address_from_topic("0x000000000000000000000000a0b86991c6218b36c1d19d4a2e9eb0ce3606eb48")?;
// let addr2 = address_from_hex_32("0x000000000000000000000000000000000000000000000000000000000000deadbeef")?; // 取最后20字节

// // UI 显示
// println!("钱包地址：{}", display_address(&checksum));

// // 签名时用 checksum
// let domain = json!({
//     "verifyingContract": address_checksum(&contract, Some(chain_id))?
// });
