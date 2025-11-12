export type HexString = `0x${string}`;

export type App = {
	readonly id: number;
	readonly name: string;
	readonly app_path: string;
	description: string;
	supported_chain_ids: number[];
};

export type Metadata = {
	readonly chainId: number;
	readonly address: HexString;
	logo_url: string;
	description: string;
	risk_level: 'low' | 'medium' | 'high';
	risk_notification: string;
};

export type Token = {
	readonly chain_id: number;
	readonly address: HexString;
	readonly name: string;
	readonly symbol: string;
	readonly decimals: number;
	logo_url: string;
	token_type: 'currency' | 'erc-20' | 'erc-777' | 'erc-1155' | '';
};

export interface Nft {
	readonly chain_id: number;
	readonly address: HexString;
	readonly name: string;
	readonly symbol: string;
	token_id?: number;
	quantity?: number;
	token_uri?: string;
	nft_type: 'erc-721' | 'erc-1155'|'';
}

export type AccountType = 'local' |'hardware'|'airgap'|'watch';
export interface Account {
	name: string;
	readonly address: string;
	readonly account_index: number;
	readonly account_type: AccountType;
	readonly derive_path?: string;
	avatar?: string;
	memo?: string;
	ens?: string;
	nft?: Nft;
	created_at: number; // unix timestamp seconds
	isHidden: boolean;
}


export interface AddressEntry {
	address: string
	name: string
	addr_type: string
	ens?: string
	nft?: string
	memo?: string
	avatar?: string // emoji，用 string 即可
  }
  
  
  export interface TransactionHistoryEntry {
	block_hash: string
	block_number: string
	from: string
	to: string
	value: string
	input: string
	nonce: number
	gas: number
	gas_price: number
	gas_used: number
	r: string
	s: string
	v: string
	status:  'Pending' | 'Confirmed' | 'Failed'
	txtype: string
  }
  
  export interface MessageHistoryEntry {
	chain: string
	msg_type: "191" | "712"
	signer: string
	msg_hash: string
	msg: any
	signature?: string
	timestamp: number
	status?: string
  }

export interface CustomRpc {
    chain: string;
    rpc_type: string;
    endpoint: string;
}


// cryptoVersion: V1;
// kdf: 'scrypt(password, salt, { N: 2 ** 16, r: 8, p: 1, dkLen: 32 })'
// symmetric: 'XChaCha20-Poly1305-managedNonce'


export interface Chain {
	chainId: number;
	name: string;
	nativeCurrency: {
		name: string;
		symbol: string;
		decimals: number;
	};
	blockExplorers: string;
	multicall3?: {
		address: string;
		blockCreated: number;
	};
	testnet: boolean;
}

export type Fiat = {
	name: string;
	symbol: string;
}

export interface FiatRate {
	timestamp: number;
	EUR:number;
	GBP:number;
	JPY:number;
	CNY:number;
	KRW:number;
	RUB:number;
}

export interface CurrencyPrice {
	timestamp: number;
	BTC:number;
	ETH:number;
	DOT:number;
}




export interface UiState {
  locale?: string;
  dark_mode?: boolean;
  current_account_index?: number;
  next_account_index?: number;
  next_watch_account_index?: number;
  next_airgap_account_index?: number;
  next_hdwallet_account_index?: number;
  auto_lock?: boolean;
  auto_lock_timer?: number; 
  active_apps?: App[]|null;
  hidden_apps?: App[]|null;
  currency?: string;
  fiat?: string;
  is_initialized?: boolean;
  is_keystore_backuped?: boolean;
}

export const DefaultUiConfig: UiState= {
  locale: 'en',
  dark_mode: false,
  current_account_index: 0,
  next_account_index: 1,
  next_watch_account_index: 101,
  next_airgap_account_index: 201,
  next_hdwallet_account_index: 301,
  auto_lock: true,
  auto_lock_timer: 900,
  active_apps: null,
  hidden_apps: null,
  currency: "ETH",
  fiat: "USD",
  is_initialized: false,
  is_keystore_backuped: false,
};