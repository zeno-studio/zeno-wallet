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


// ok
export interface CurrencyPrice {
	timestamp: number;
	BTC:number;
	ETH:number;
	BNB:number;
	POL:number;
}



export interface FiatRates {
  timestamp: number;

  USD: number;
  EUR: number;
  GBP: number;
  JPY: number;
  CNY: number;
  KRW: number;

  SGD: number;
  VND: number;
  MYR: number;
  IDR: number;
  THB: number;
  PHP: number;

  INR: number;
  PKR: number;

  VES: number;
  ARS: number;
  BRL: number;
  CLP: number;
  COP: number;
  PEN: number;

  CHF: number;
  CAD: number;
  AUD: number;
  NZD: number;
}

export type FiatCode = keyof FiatRates;

export interface Locales {
  "zh-CN": boolean;
  "zh-TW": boolean;
  "ja-JP": boolean;
  "ko-KR": boolean;

  "en-SG": boolean;
  "vi-VN": boolean;
  "ms-MY": boolean;
  "id-ID": boolean;

  "en-IN": boolean;
  "hi-IN": boolean;

  "es-AR": boolean;
  "es-VE": boolean;
  "pt-BR": boolean;

  "en-US": boolean;
  "es-ES": boolean;
  "fr-FR": boolean;
  "de-DE": boolean;
  "ru-RU": boolean;
}

export type Locale = keyof Locales;
