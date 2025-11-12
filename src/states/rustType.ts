
export interface Vault {
	readonly Version: string;
	readonly salt: string;
	readonly nonce: string;
	readonly ciphertext: string;

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

export interface Account {
  index: string
  address: string
  name: string
  created_at: number
  addr_type: string
  is_hidden: boolean
  is_locked: boolean
  ens?: string
  nft?: string
  memo?: string
  avatar?: string
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
  status: string
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
