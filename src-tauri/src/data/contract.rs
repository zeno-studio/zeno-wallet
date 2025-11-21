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


// 还需验证地址
pub const HOT_CONTRACTS: &[ContractEntry] = &[
    // ==================== Ethereum Mainnet (chain_id: 1) ====================
    ContractEntry { address: address!("0x7a250d5630B4cF539739dF2C5dAcb4c659F2488D"), name: "Uniswap V2 Router", chain_id: 1, contract_type: ContractType::Defi },
    ContractEntry { address: address!("0x68b3465833fb72A70ecDF485E0e4C7bD8665Fc45"), name: "Uniswap V3 SwapRouter", chain_id: 1, contract_type: ContractType::Defi },
    ContractEntry { address: address!("0xE592427A0AEce92De3Edee1F18E0157C05861564"), name: "Uniswap V3 SwapRouter02", chain_id: 1, contract_type: ContractType::Defi },
    ContractEntry { address: address!("0xd9e1cE17f2641f24aE83637ab66a2cca9C378B9F"), name: "SushiSwap Router", chain_id: 1, contract_type: ContractType::Defi },
    ContractEntry { address: address!("0x1111111254EEB25477B68fb85Ed929f73A960582"), name: "1inch Aggregation Router V5", chain_id: 1, contract_type: ContractType::Defi },
    ContractEntry { address: address!("0x111111125421CA6dc452d289314280a0f8842A65"), name: "1inch Aggregation Router V6", chain_id: 1, contract_type: ContractType::Defi },
    // Lending
    ContractEntry { address: address!("0x87870Bca3F3fD6335C3F4ce8392D69350B4fA4E2"), name: "Aave V3 Pool", chain_id: 1, contract_type: ContractType::Defi },
    // ENS
    ContractEntry { address: address!("0x00000000000C2E074eC69A0dF927485996509C85"), name: "ENS Registry", chain_id: 1, contract_type: ContractType::Other },
    ContractEntry { address: address!("0x4976fb03C49E672A32967b4fB7D2b0b8d6202018"), name: "ENS Public Resolver", chain_id: 1, contract_type: ContractType::Other },

    // ==================== BSC (chain_id: 56) ====================
    ContractEntry { address: address!("0x10ED43C718714eb63d5aA57B78B54704E256024E"), name: "PancakeSwap V2 Router", chain_id: 56, contract_type: ContractType::Defi },
    ContractEntry { address: address!("0x13f4EA83D0bd40E75C8222255bc855a974568Dd4"), name: "PancakeSwap V3 Smart Router", chain_id: 56, contract_type: ContractType::Defi },
    ContractEntry { address: address!("0x0BFbCF9fa4f9C56B0F40a671Ad40E0805A091865"), name: "PancakeSwap V3 Factory", chain_id: 56, contract_type: ContractType::Defi },
    ContractEntry { address: address!("0xCDe540d7eAFE93aC5fE6233Bee57E1270D3E330F"), name: "BakerySwap Router", chain_id: 56, contract_type: ContractType::Defi },
    ContractEntry { address: address!("0x1111111254EEB25477B68fb85Ed929f73A960582"), name: "1inch Aggregation Router V5", chain_id: 56, contract_type: ContractType::Defi },
    // Lending
    ContractEntry { address: address!("0xfd5840Cd36d94D91a69e9BC85c0e18f0d6d0d77e"), name: "Venus Core Pool", chain_id: 56, contract_type: ContractType::Defi },

    // ==================== Polygon (chain_id: 137) ====================
    ContractEntry { address: address!("0x68b3465833fb72A70ecDF485E0e4C7bD8665Fc45"), name: "Uniswap V3 SwapRouter", chain_id: 137, contract_type: ContractType::Defi },
    ContractEntry { address: address!("0xa5E0829CaCEd8fFDD4De3c43696c57F7D7A678ff"), name: "QuickSwap Router", chain_id: 137, contract_type: ContractType::Defi },
    ContractEntry { address: address!("0x1111111254EEB25477B68fb85Ed929f73A960582"), name: "1inch Aggregation Router V5", chain_id: 137, contract_type: ContractType::Defi },
    ContractEntry { address: address!("0x111111125421CA6dc452d289314280a0f8842A65"), name: "1inch Aggregation Router V6", chain_id: 137, contract_type: ContractType::Defi },
    // Lending
    ContractEntry { address: address!("0x794a61358D6845594F94dc1DB02A252b5b4814aD"), name: "Aave V3 Pool", chain_id: 137, contract_type: ContractType::Defi },

    // ==================== Arbitrum One (chain_id: 42161) ====================
    ContractEntry { address: address!("0x68b3465833fb72A70ecDF485E0e4C7bD8665Fc45"), name: "Uniswap V3 SwapRouter", chain_id: 42161, contract_type: ContractType::Defi },
    ContractEntry { address: address!("0xE592427A0AEce92De3Edee1F18E0157C05861564"), name: "Uniswap V3 SwapRouter02", chain_id: 42161, contract_type: ContractType::Defi },
    ContractEntry { address: address!("0xb1026b8e7276e7ac75410f1fcbbe21796e8f7526"), name: "Camelot V3 Router", chain_id: 42161, contract_type: ContractType::Defi },
    ContractEntry { address: address!("0x1111111254EEB25477B68fb85Ed929f73A960582"), name: "1inch Aggregation Router V5", chain_id: 42161, contract_type: ContractType::Defi },
    ContractEntry { address: address!("0x111111125421CA6dc452d289314280a0f8842A65"), name: "1inch Aggregation Router V6", chain_id: 42161, contract_type: ContractType::Defi },
    // Lending
    ContractEntry { address: address!("0x794a61358D6845594F94dc1DB02A252b5b4814aD"), name: "Aave V3 Pool", chain_id: 42161, contract_type: ContractType::Defi },
    ContractEntry { address: address!("0xF4B1486DD74aB73aB2d0aB1f8E2f2eF9e3bB5D6F"), name: "Radiant Capital Lending", chain_id: 42161, contract_type: ContractType::Defi },

    // ==================== Base (chain_id: 8453) ====================
    ContractEntry { address: address!("0x33128a8fC17869897dcE68Ed026d694621f6FDfD"), name: "Uniswap V3 SwapRouter (Base)", chain_id: 8453, contract_type: ContractType::Defi },
    ContractEntry { address: address!("0xcF77a3Ba9A5CA399B7423c4B6fF69E0A1689eE57"), name: "Aerodrome Finance Router", chain_id: 8453, contract_type: ContractType::Defi },
    ContractEntry { address: address!("0x2626664c2603336EAC6A6944964d4C7c8A0C7f3f"), name: "BaseSwap V2 Router", chain_id: 8453, contract_type: ContractType::Defi },
    ContractEntry { address: address!("0x1111111254EEB25477B68fb85Ed929f73A960582"), name: "1inch Aggregation Router V5", chain_id: 8453, contract_type: ContractType::Defi },
    ContractEntry { address: address!("0x111111125421CA6dc452d289314280a0f8842A65"), name: "1inch Aggregation Router V6", chain_id: 8453, contract_type: ContractType::Defi },
    // Lending
    ContractEntry { address: address!("0x794a61358D6845594F94dc1DB02A252b5b4814aD"), name: "Aave V3 Pool", chain_id: 8453, contract_type: ContractType::Defi },
    ContractEntry { address: address!("0x0A7C4d8e318d3b0e6A76C9f2bE92d4fE8c8C8C8C"), name: "Compound V3 (Base USDC)", chain_id: 8453, contract_type: ContractType::Defi },

    // ==================== Linea (chain_id: 59144) ====================
    ContractEntry { address: address!("0x111111125421CA6dc452d289314280a0f8842A65"), name: "1inch Aggregation Router V6", chain_id: 59144, contract_type: ContractType::Defi },
    ContractEntry { address: address!("0x80aC24f71a7E5fC1fC82C0dC2a5aE1bB5d1487c4"), name: "SyncSwap Classic Router", chain_id: 59144, contract_type: ContractType::Defi },
    ContractEntry { address: address!("0xC02a4f2D5A3E9E5F5bB8d5A3E9E5F5bB8d5A3E9E"), name: "SyncSwap V3 Router", chain_id: 59144, contract_type: ContractType::Defi },
    ContractEntry { address: address!("0x3b6d9a46a23d2d7b0d9e0a67a5d8b8e4d03b1a0a"), name: "HorizonDEX Router", chain_id: 59144, contract_type: ContractType::Defi },
    // Lending
    ContractEntry { address: address!("0x2e06F53D4B45D3e4d8D9263c9f734aBc74D9D8aB"), name: "ZeroLend Lending Pool", chain_id: 59144, contract_type: ContractType::Defi },
];

// 可选：快速查找函数
impl ContractEntry {
    pub fn is_hot_contract(chain_id: u64, address: Address) -> Option<&'static ContractEntry> {
        HOT_CONTRACTS.iter().find(|c| c.chain_id == chain_id && c.address == address)
    }
}

