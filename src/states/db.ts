import { invoke } from '@tauri-apps/api/core'
import type {
  Vault,
  AddressEntry,
  TransactionHistoryEntry,
  MessageHistoryEntry,
  Account,
} from './rustType'

export const db = {
  // ========= Config =========
  async getConfig(key: string): Promise<any> {
    return await invoke('config_get', { key })
  },

  async setConfig(key: string, value: any): Promise<void> {
    await invoke('config_set', { key, value })
  },

  // ========= Vault =========
  async getVault(key: string): Promise<Vault> {
    return await invoke('vault_get', { key })
  },

  async setVault(key: string, v: Vault): Promise<void> {
    await invoke('vault_set', { key, v })
  },

  // ========= AddressBook =========
  async getAddress(category: string, id: string): Promise<AddressEntry> {
    return await invoke('addr_book_get', { category, id })
  },

  async listAddress(category: string): Promise<[string, AddressEntry][]> {
    return await invoke('addr_book_list', { category })
  },

  async setAddress(category: string, id: string, v: AddressEntry): Promise<void> {
    await invoke('addr_book_set', { category, id, v })
  },

  // ========= Transaction =========
  async addTx(chain: string, ts: number, id: string, item: TransactionHistoryEntry): Promise<void> {
    await invoke('tx_add', { chain, ts, id, item })
  },

  async listTx(
    chain: string,
    since?: number,
    until?: number,
  ): Promise<TransactionHistoryEntry[]> {
    return await invoke('tx_list', { chain, since, until })
  },

  async findTx(chain: string, id: string): Promise<TransactionHistoryEntry | null> {
    return await invoke('tx_find', { chain, id })
  },

  async deleteTx(chain: string, id: string): Promise<void> {
    await invoke('tx_delete', { chain, id })
  },

  // ========= Message =========
  async setMessage(hash: string, value: MessageHistoryEntry): Promise<void> {
    await invoke('message_set', { hash, value })
  },

  async listMessage(): Promise<MessageHistoryEntry[]> {
    return await invoke('message_list')
  },

  // ========= Account =========
  async getAccount(index: string): Promise<Account> {
    return await invoke('account_get', { index })
  },

  async setAccount(index: string, value: Account): Promise<void> {
    await invoke('account_set', { index, value })
  },

  async listAccount(): Promise<Account[]> {
    return await invoke('account_list')
  },
}
