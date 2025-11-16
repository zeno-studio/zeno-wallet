

use alloy_primitives::{U256};

use crate::error::AppError;

pub fn parse_eth_to_wei(eth: &str) -> Result<U256, AppError> {
    let parts: Vec<&str> = eth.split('.').collect();
    if parts.len() > 2 {
        return Err("Invalid ETH amount".to_string());
    }

    let integer = parts[0]
        .parse::<u128>()
        .map_err(|_| "Invalid integer part")?;
    let fractional = parts.get(1).map(|s| *s).unwrap_or("");

    let mut wei = U256::from(integer) * U256::from(10u64.pow(18));
    if !fractional.is_empty() {
        let frac_str = fractional.trim_end_matches('0');
        if frac_str.len() > 18 {
            return Err("Too many decimal places".to_string());
        }
        let frac: u128 = frac_str.parse().map_err(|_| "Invalid fractional part")?;
        let decimals = frac_str.len() as u32;
        wei += U256::from(frac) * U256::from(10u64.pow(18 - decimals));
    }

    Ok(wei)
}

#[tauri::command]
pub fn convert_eth_to_wei(eth: &str) -> Result<String, String> {
    parse_eth_to_wei(eth).map(|wei| wei.to_string())
}
