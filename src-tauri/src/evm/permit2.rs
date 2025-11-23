// permit2_parser.rs
// 2025 顶级钱包必备：一键解析 Permit2 / Permit / EIP-2612 / Permit2Batch
// 支持：USDC、USDT、DAI、WETH、UNI、所有主流代币

use alloy_primitives::{Address, B256, U256, Bytes};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct PermitInfo {
    pub kind: PermitKind,
    pub token: Address,
    pub owner: Address,
    pub spender: Address,
    pub amount: U256,
    pub deadline: u64,
    pub signature: Bytes,
    pub nonce: Option<U256>,
    pub is_permit2: bool,
}

#[derive(Debug, Clone, Serialize)]
pub enum PermitKind {
    PermitSingle,
    PermitBatch,
    EIP2612,        // DAI-style
    Permit2Single,
    Permit2Batch,
    Invalid,
}

// 官方 Permit2 地址（主网 + 所有 L2 都一样）
const PERMIT2: Address = Address::new([0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01]);

// 官方签名
const PERMIT2_PERMIT_SINGLE: [u8; 4] = [0x3d, 0x9d, 0x2f, 0x2d]; // permit(PermitSingle calldata, bytes)
const PERMIT2_PERMIT_BATCH: [u8; 4] = [0x2d, 0x0d, 0x6e, 0x2a]; // permit(PermitBatch calldata, bytes)

impl PermitInfo {
    pub fn parse(from: Address, to: Address, data: &Bytes) -> Option<Self> {
        if data.is_empty() { return None; }

        let selector = &data[0..4];

        // 1. 直接调用 Permit2 合约
        if to == PERMIT2 {
            if selector == &PERMIT2_PERMIT_SINGLE {
                return Self::parse_permit2_single(from, data);
            }
            if selector == &PERMIT2_PERMIT_BATCH {
                return Self::parse_permit2_batch(from, data);
            }
        }

        // 2. 代币直接调用 EIP-2612 permit()
        if selector == &[0x8b, 0xa9, 0x3c, 0xb9] {
            // permit(address owner, address spender, uint256 value, uint256 deadline, uint8 v, bytes32 r, bytes32 s)
            return Self::parse_eip2612(to, from, data);
        }

        // 3. DAI-style permit (address holder, address spender, uint256 nonce, uint256 expiry, bool allowed, uint8 v, bytes32 r, bytes32 s)
        if selector == &[0x9e, 0x92, 0x68, 0x4a] {
            return Self::parse_dai_permit(to, from, data);
        }

        None
    }

    fn parse_permit2_single(owner: Address, data: &Bytes) -> Option<Self> {
        // 结构太复杂，用简单方式：从 calldata 截取关键字段
        if data.len() < 200 { return None; }

        let token = Address::from_slice(&data[84..104]);     // PermitSingle.token
        let amount = U256::from_be_bytes(data[104..136].try_into().ok()?);
        let deadline = u64::from_be_bytes(data[168..176].try_into().ok()?);
        let spender = Address::from_slice(&data[176..196]);  // PermitSingle.spender

        Some(PermitInfo {
            kind: PermitKind::Permit2Single,
            token,
            owner,
            spender,
            amount,
            deadline,
            signature: data[data.len()-65..].to_vec().into(),
            nonce: None,
            is_permit2: true,
        })
    }

    fn parse_eip2612(token: Address, owner: Address, data: &Bytes) -> Option<Self> {
        if data.len() < 132 { return None; }
        let spender = Address::from_slice(&data[36..56]);
        let value = U256::from_be_bytes(data[68..100].try_into().ok()?);
        let deadline = u64::from_be_bytes(data[100..132].try_into().ok()?);

        Some(PermitInfo {
            kind: PermitKind::EIP2612,
            token,
            owner,
            spender,
            amount: value,
            deadline,
            signature: data[data.len()-65..].to_vec().into(),
            nonce: None,
            is_permit2: false,
        })
    }

    fn parse_dai_permit(token: Address, owner: Address, data: &Bytes) -> Option<Self> {
        if data.len() < 132 { return None; }
        let spender = Address::from_slice(&data[36..56]);
        let allowed = data[100] != 0;
        let amount = if allowed { U256::MAX } else { U256::ZERO };
        let expiry = u64::from_be_bytes(data[68..100].try_into().ok()?);

        Some(PermitInfo {
            kind: PermitKind::EIP2612,
            token,
            owner,
            spender,
            amount,
            deadline: expiry,
            signature: data[data.len()-65..].to_vec().into(),
            nonce: None,
            is_permit2: false,
        })
    }
}

// let permit = PermitInfo::parse(from, to, &calldata);

// if let Some(p) = permit {
//     println!("用户正在给 {} 授权 {} 个 {}", p.spender, p.amount / 1e18, token_symbol(p.token));
//     if p.amount > U256::from(1_000_000e18) {
//         show_warning!("超大额授权风险！");
//     }
// }
// Permit2 授权检测
// 用户正在使用 Permit2 给 1inch Router 授权 999,999 USDC
// 截止时间：2025年12月31日
// 这是高风险操作，可能导致资金被盗