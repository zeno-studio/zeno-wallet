

pub struct chain {
	chain_id: u64,
	name: String,
    currency_name: String,
    currency_symbol: String,
    decimals: u64,
	block_explorers: String,
}

pub const SUPPORTED_CHAINS: [chain; 2] = [
	chain {
		chain_id: 1,
		name: "Ethereum".to_string(),
		currency_name: "Ether".to_string(),
		currency_symbol: "ETH".to_string(),
		decimals: 18,
		block_explorers: "https://etherscan.io/".to_string(),
	},
	chain {
		chain_id: 56,
		name: "Binance Smart Chain".to_string(),
		currency_name: "Binance Coin".to_string(),
		currency_symbol: "BNB".to_string(),
		decimals: 18,
		block_explorers: "https://bscscan.com/".to_string(),
	},
    chain {
		chain_id: 137,
		name: "Polygon Mainnet".to_string(),
		currency_name: "Polygon".to_string(),
		currency_symbol: "POL".to_string(),
		decimals: 18,
		block_explorers: "https://polygonscan.com/".to_string(),
	},
    chain {
		chain_id: 8453,
        name: "Base".to_string(),
		currency_name: "Ether".to_string(),
		currency_symbol: "ETH".to_string(),
		decimals: 18,
		block_explorers: "https://basescan.org/".to_string(),
	},
    chain {
		chain_id: 42161,
		name: "Arbitrum One".to_string(),
		currency_name: "Ether".to_string(),
		currency_symbol: "ETH".to_string(),
		decimals: 18,
		block_explorers: "https://arbiscan.io/".to_string(),
	},
    chain {
		chain_id:59144,
        name: "Linea".to_string(),
		currency_name: "Ether".to_string(),
		currency_symbol: "ETH".to_string(),
		decimals: 18,
		block_explorers: "https://lineascan.build/".to_string(),
	},
    chain {
		chain_id: 11155111,
		name: "Sepolia".to_string(),
		currency_name: "Ether".to_string(),
		currency_symbol: "ETH".to_string(),
		decimals: 18,
		block_explorers: "https://sepolia.etherscan.io/".to_string(),
	},
];
