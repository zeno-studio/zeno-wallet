use alloy_primitives::{address, Address};

#[derive(Debug, Clone)]
pub struct ContractEntry {
    pub address: Address,
    pub name: &'static str,
    pub chain_id: u64,           // 直接用 u64，告别 Chain 枚举
    pub contract_type: ContractType,
}

#[derive(Debug, Clone)]
pub enum ContractType {
    Erc20,
    Erc721,
    Erc1155,
    Defi,
    DAO,
    Gaming,
    Other,
}

pub const HOT_CONTRACTS: &[ContractEntry] = &[
    // ==================== Ethereum Mainnet (chain_id: 1) ====================
    ContractEntry { address: address!("0x7a250d5630B4cF539739dF2C5dAcb4c659F2488D"), name: "Uniswap V2 Router", chain_id: 1, contract_type: ContractType::Defi },
    ContractEntry { address: address!("0x68b3465833fb72A70ecDF485E0e4C7bD8665Fc45"), name: "Uniswap V3 Router", chain_id: 1, contract_type: ContractType::Defi },
    ContractEntry { address: address!("0xE592427A0AEce92De3Edee1F18E0157C05861564"), name: "Uniswap V3 SwapRouter02", chain_id: 1, contract_type: ContractType::Defi },
    ContractEntry { address: address!("0x1111111254EEB25477B68fb85Ed929f73A960582"), name: "1inch Aggregation Router V5", chain_id: 1, contract_type: ContractType::Defi },
    ContractEntry { address: address!("0x111111125421CA6dc452d289314280a0f8842A65"), name: "1inch Aggregation Router V6", chain_id: 1, contract_type: ContractType::Defi },

    // ==================== BSC (chain_id: 56) ====================
    ContractEntry { address: address!("0x10ED43C718714eb63d5aA57B78B54704E256024E"), name: "PancakeSwap V2 Router", chain_id: 56, contract_type: ContractType::Defi },
    ContractEntry { address: address!("0x13f4EA83D0bd40E75C8222255bc855a974568Dd4"), name: "PancakeSwap V3 MasterChef", chain_id: 56, contract_type: ContractType::Defi },
    ContractEntry { address: address!("0x172fcD41e0913e95784454622d1c3724f546f849"), name: "PancakeSwap V3 Factory", chain_id: 56, contract_type: ContractType::Defi },
    ContractEntry { address: address!("0x0BFbCF9fa4f9C56B0F40a671Ad40E0805A091865"), name: "PancakeSwap V3 Router", chain_id: 56, contract_type: ContractType::Defi },
    ContractEntry { address: address!("0x1111111254EEB25477B68fb85Ed929f73A960582"), name: "1inch Aggregation Router V5", chain_id: 56, contract_type: ContractType::Defi },

    // ==================== Base (chain_id: 8453) ====================
    ContractEntry { address: address!("0x33128a8fC17869897dcE68Ed026d694621f6FDfD"), name: "Uniswap V3 Router (Base)", chain_id: 8453, contract_type: ContractType::Defi },
    ContractEntry { address: address!("0x2626664c2603336EAC6A6944964d4C7c8A0C7f3f"), name: "BaseSwap Router", chain_id: 8453, contract_type: ContractType::Defi },
    ContractEntry { address: address!("0x1111111254EEB25477B68fb85Ed929f73A960582"), name: "1inch Aggregation Router V5", chain_id: 8453, contract_type: ContractType::Defi },
    ContractEntry { address: address!("0x111111125421CA6dc452d289314280a0f8842A65"), name: "1inch Aggregation Router V6", chain_id: 8453, contract_type: ContractType::Defi },

    // ==================== Arbitrum One (chain_id: 42161) ====================
    ContractEntry { address: address!("0x68b3465833fb72A70ecDF485E0e4C7bD8665Fc45"), name: "Uniswap V3 Router", chain_id: 42161, contract_type: ContractType::Defi },
    ContractEntry { address: address!("0x1111111254EEB25477B68fb85Ed929f73A960582"), name: "1inch Aggregation Router V5", chain_id: 42161, contract_type: ContractType::Defi },
    ContractEntry { address: address!("0x111111125421CA6dc452d289314280a0f8842A65"), name: "1inch Aggregation Router V6", chain_id: 42161, contract_type: ContractType::Defi },
    ContractEntry { address: address!("0xb1026b8e7276e7ac75410f1fcbbe21796e8f7526"), name: "Camelot V3 Router", chain_id: 42161, contract_type: ContractType::Defi },

    // ==================== Linea (chain_id: 59144) ====================
    ContractEntry { address: address!("0x111111125421CA6dc452d289314280a0f8842A65"), name: "1inch Aggregation Router V6", chain_id: 59144, contract_type: ContractType::Defi },
    ContractEntry { address: address!("0x3b6d9a46a23d2d7b0d9e0a67a5d8b8e4d03b1a0a"), name: "LineaSwap Router", chain_id: 59144, contract_type: ContractType::Defi },
    ContractEntry { address: address!("0x9aB3e1541d6E8f8e6cB3d6e7F8F6eE6D6f6E6d6f"), name: "SyncSwap Router", chain_id: 59144, contract_type: ContractType::Defi },
];

// 可选：快速查找函数
impl ContractEntry {
    pub fn is_hot_contract(chain_id: u64, address: Address) -> Option<&'static ContractEntry> {
        HOT_CONTRACTS.iter().find(|c| c.chain_id == chain_id && c.address == address)
    }
}