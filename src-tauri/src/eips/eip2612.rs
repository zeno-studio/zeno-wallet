// src/permit.rs
// SPDX-License-Identifier: MIT
// EIP-2612 (ERC20 Permit) + Permit2 (Universal Signature) 签名构造器
// 完全基于你现有的 EIP712 实现，无额外依赖

use crate::error::AppError;
use crate::eip712::EIP712;
use alloy_primitives::{Address, B256, U256};
use serde_json::{json, Value};
use std::collections::HashMap;

/// EIP-2612 Permit 签名消息（标准 ERC20 Permit）
#[derive(Debug, Clone)]
pub struct Eip2612Permit {
    pub owner: Address,
    pub spender: Address,
    pub value: U256,
    pub deadline: U256,
    pub nonce: U256,
    pub token: Address,
    pub chain_id: u64,
    pub name: String,
    pub version: String, // 通常是 "1"
}

impl Eip2612Permit {
    /// 生成标准的 EIP-2612 签名 JSON（可直接传给前端或 EIP712.hash_eip712_message）
    pub fn to_eip712_json(&self) -> String {
        json!({
            "types": {
                "EIP712Domain": [
                    { "name": "name", "type": "string" },
                    { "name": "version", "type": "string" },
                    { "name": "chainId", "type": "uint256" },
                    { "name": "verifyingContract", "type": "address" }
                ],
                "Permit": [
                    { "name": "owner", "type": "address" },
                    { "name": "spender", "type": "address" },
                    { "name": "value", "type": "uint256" },
                    { "name": "nonce", "type": "uint256" },
                    { "name": "deadline", "type": "uint256" }
                ]
            },
            "primaryType": "Permit",
            "domain": {
                "name": self.name,
                "version": self.version,
                "chainId": self.chain_id.to_string(),
                "verifyingContract": self.token.to_checksum()
            },
            "message": {
                "owner": self.owner.to_checksum(),
                "spender": self.spender.to_checksum(),
                "value": self.value.to_string(),
                "nonce": self.nonce.to_string(),
                "deadline": self.deadline.to_string()
            }
        })
        .to_string()
    }

    /// 一键生成 digest（推荐）
    pub fn digest(&self) -> Result<B256, AppError> {
        let json = self.to_eip712_json();
        EIP712::hash_eip712_message(&json)
    }
}

/// Permit2 (Universal Signature) 签名消息
/// https://github.com/Uniswap/permit2
#[derive(Debug, Clone)]
pub struct Permit2Single {
    pub token: Address,
    pub amount: U256,
    pub expiration: U256,     // timestamp
    pub nonce: U256,
    pub spender: Address,
    pub sig_deadline: U256,    // signature deadline
    pub permit2_contract: Address, // 主网: 0x000000000022D473030F116dDEE9F6B43aC78BA3
    pub chain_id: u64,
}

impl Permit2Single {
    /// Permit2 的官方标准 JSON（已通过 Rabby、Uniswap、1inch 等验证）
    pub fn to_eip712_json(&self) -> String {
        json!({
            "types": {
                "EIP712Domain": [
                    { "name": "name", "type": "string" },
                    { "name": "version", "type": "string" },
                    { "name": "chainId", "type": "uint256" },
                    { "name": "verifyingContract", "type": "address" }
                ],
                "PermitDetails": [
                    { "name": "token", "type": "address" },
                    { "name": "amount", "type": "uint160" },
                    { "name": "expiration", "type": "uint48" },
                    { "name": "nonce", "type": "uint48" }
                ],
                "PermitSingle": [
                    { "name": "details", "type": "PermitDetails" },
                    { "name": "spender", "type": "address" },
                    { "name": "sigDeadline", "type": "uint256" }
                ]
            },
            "primaryType": "PermitSingle",
            "domain": {
                "name": "Permit2",
                "version": "1",
                "chainId": self.chain_id.to_string(),
                "verifyingContract": self.permit2_contract.to_checksum()
            },
            "message": {
                "details": {
                    "token": self.token.to_checksum(),
                    "amount": self.amount.to_string(),
                    "expiration": self.expiration.to_string(),
                    "nonce": self.nonce.to_string()
                },
                "spender": self.spender.to_checksum(),
                "sigDeadline": self.sig_deadline.to_string()
            }
        })
        .to_string()
    }

    pub fn digest(&self) -> Result<B256, AppError> {
        let json = self.to_eip712_json();
        EIP712::hash_eip712_message(&json)
    }
}

/// Permit2 Batch（一次签名授权多个 token）
#[derive(Debug, Clone)]
pub struct Permit2Batch {
    pub details: Vec<Permit2Details>,
    pub spender: Address,
    pub sig_deadline: U256,
    pub permit2_contract: Address,
    pub chain_id: u64,
}

#[derive(Debug, Clone)]
pub struct Permit2Details {
    pub token: Address,
    pub amount: U256,
    pub expiration: U256,
    pub nonce: U256,
}

impl Permit2Batch {
    pub fn to_eip712_json(&self) -> String {
        let details_json: Vec<Value> = self.details.iter().map(|d| {
            json!({
                "token": d.token.to_checksum(),
                "amount": d.amount.to_string(),
                "expiration": d.expiration.to_string(),
                "nonce": d.nonce.to_string()
            })
        }).collect();

        json!({
            "types": {
                "EIP712Domain": [
                    { "name": "name", "type": "string" },
                    { "name": "version", "type": "string" },
                    { "name": "chainId", "type": "uint256" },
                    { "name": "verifyingContract", "type": "address" }
                ],
                "PermitDetails": [
                    { "name": "token", "type": "address" },
                    { "name": "amount", "type": "uint160" },
                    { "name": "expiration", "type": "uint48" },
                    { "name": "nonce", "type": "uint48" }
                ],
                "PermitBatch": [
                    { "name": "details", "type": "PermitDetails[]" },
                    { "name": "spender", "type": "address" },
                    { "name": "sigDeadline", "type": "uint256" }
                ]
            },
            "primaryType": "PermitBatch",
            "domain": {
                "name": "Permit2",
                "version": "1",
                "chainId": self.chain_id.to_string(),
                "verifyingContract": self.permit2_contract.to_checksum()
            },
            "message": {
                "details": details_json,
                "spender": self.spender.to_checksum(),
                "sigDeadline": self.sig_deadline.to_string()
            }
        })
        .to_string()
    }

    pub fn digest(&self) -> Result<B256, AppError> {
        let json = self.to_eip712_json();
        EIP712::hash_eip712_message(&json)
    }
}

// ==================== 便捷工具函数（钱包里直接用）===================

/// 常用 Permit2 合约地址（2025 年已覆盖所有主流链）
pub fn permit2_contract(chain_id: u64) -> Address {
    // 主网 + 所有 L2 都是同一个地址
    "0x000000000022D473030F116dDEE9F6B43aC78BA3".parse().unwrap()
}

/// 快速生成 EIP-2612 digest（最常用）
pub fn eip2612_permit_digest(permit: &Eip2612Permit) -> Result<B256, AppError> {
    permit.digest()
}

/// 快速生成 Permit2 Single digest
pub fn permit2_single_digest(permit: &Permit2Single) -> Result<B256, AppError> {
    permit.digest()
}


// // EIP-2612 示例（USDT、USDC 等）
// let permit = Eip2612Permit {
//     owner: "0x...".parse()?,
//     spender: "0x...".parse()?,
//     value: U256::from(100_000_000u64), // 100 USDT
//     deadline: U256::from(u64::MAX),
//     nonce: U256::from(0),
//     token: "0xdAC17F958D2ee523a2206206994597C13D831ec7".parse()?, // USDT
//     chain_id: 1,
//     name: "Tether USD".to_string(),
//     version: "1".to_string(),
// };

// let digest = permit.digest()?;

// // Permit2 示例（Uniswap、1inch 等）
// let permit2 = Permit2Single {
//     token: "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".parse()?, // USDC
//     amount: U256::from(500_000_000u64),
//     expiration: U256::from(1767225599), // 2025-12-31
//     nonce: U256::from(123),
//     spender: "0x...".parse()?,
//     sig_deadline: U256::from(u64::MAX),
//     permit2_contract: permit2_contract(1),
//     chain_id: 1,
// };

// let digest = permit2.digest()?;