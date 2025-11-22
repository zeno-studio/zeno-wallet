use serde::{Deserialize, Serialize};
use tauri::State;
use bincode::{Decode, Encode};
use crate::core::db::{AppDB,TxHistoryManager};
use crate::error::AppError;
use alloy_primitives::{U256, U128, Address,TxHash};


#[derive(Debug, Serialize, Deserialize, Clone, Encode, Decode, PartialEq)]
pub struct TransactionHistoryEntry {
    pub chain_id: u64,
    pub hash: TxHash,
    pub block_number: u64,
    pub nonce: U256,
    pub from: Address,
    pub to: Address,
    pub value: U256,
    pub gas_price: Option<U256>,
    pub gas_used: Option<U256>,
    pub timestamp: Option<u64>,
}
pub trait IntoInterTx{
    fn into_inter(self) -> TransactionHistoryEntry;
}


// ========== Transaction History ==========
#[tauri::command]
pub fn tx_list(
    chain_id: u64,
    from: Option<u64>,
    to: Option<u64>,
    appdb: State<AppDB>,
) -> Result<Vec<TransactionHistoryEntry>, AppError> {
    let db = appdb.db.as_ref();
    let mgr = TxHistoryManager::new(db);
    mgr.range(chain_id, from, to)
}

#[tauri::command]
pub fn tx_add(entry: TransactionHistoryEntry, appdb: State<AppDB>) -> Result<(), AppError> {
    let db = appdb.db.as_ref();
    let mgr = TxHistoryManager::new(db);
    mgr.insert(&entry)
}

#[tauri::command]
pub fn tx_find(
    chain_id: u64,
    hash: String,
    appdb: State<AppDB>,
) -> Result<Option<TransactionHistoryEntry>, AppError> {
    let db = appdb.db.as_ref();
    let mgr = TxHistoryManager::new(db);
    mgr.find(chain_id, &hash)
}

#[tauri::command]
pub fn tx_delete(chain_id: u64, hash: String, appdb: State<AppDB>) -> Result<(), AppError> {
    let db = appdb.db.as_ref();
    let mgr = TxHistoryManager::new(db);
    mgr.delete(chain_id, &hash)
}

#[tauri::command]
pub fn tx_batch_insert(
    items: Vec<TransactionHistoryEntry>,
    appdb: State<AppDB>,
) -> Result<(), AppError> {
    let db = appdb.db.as_ref();
    let mgr = TxHistoryManager::new(db);
    mgr.batch_insert(&items)
}

#[tauri::command]
pub fn tx_batch_delete(
    chain_id: u64,
    hashs: Vec<String>,
    appdb: State<AppDB>,
) -> Result<(), AppError> {
    let db = appdb.db.as_ref();
    let mgr = TxHistoryManager::new(db);
    mgr.batch_delete(chain_id, &hashs)
}



#[derive(Debug, Serialize, Deserialize, Clone, Encode, Decode, PartialEq)]
pub struct PendingTx {
    pub ts: i64,                    // timestamp millis → 天然全局唯一 ID（微秒更佳）
    pub chain_id: u64,
    pub from: Address,
    pub nonce: U256,   
    pub rlp: Vec<u8>,               // 原始 signed rlp，掉单后可重放
    pub to: Option<Address>,
    pub value: U256,
    pub data: Vec<u8>,
    pub status: TxStatus,           // Pending / Confirmed / Dropped / Failed
}


#[derive(Debug, Serialize, Deserialize, Clone, Encode, Decode, PartialEq)]
pub enum TxStatus {
    Pending,
    Dropped,   
    Failed,
}

type PendingDB = Arc<RwLock<DB>>;

pub struct TxTracker {
}

impl TxTracker {
    // 发送交易后立即落库（还没广播都先存）
    pub async fn add_pending(&self, tx: PendingTx) -> i64 {
        let ts = tx.ts;
        let key = ts.to_be_bytes().to_vec();
        let value = bincode::serialize(&tx).unwrap();
        self.db.write().await.put(&key, &value).unwrap();
        ts
    }

    // 广播成功拿到 hash 后更新
    pub async fn update_hash(&self, ts: i64, hash: TxHash) {
        let key = ts.to_be_bytes().to_vec();
        if let Some(mut tx) = self.get_by_ts(ts).await {
            tx.tx_hash = Some(hash);
            tx.status = TxStatus::Pending;
            self.db.write().await.put(key, &bincode::serialize(&tx).unwrap()).unwrap();
        }
    }

    // 双轮询主循环（每 6 秒跑一次）
    pub async fn start_polling(&self) {
        let tracker = self.clone();
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(6)).await;
                if let Err(e) = tracker.poll_once().await {
                    eprintln!("poll error: {}", e);
                }
            }
        });
    }

    async fn poll_once(&self) -> anyhow::Result<()> {
        let mut db = self.db.write().await;
        let iter = db.iterator(rocksdb::IteratorMode::Start);
        let mut to_delete = vec![];

        for item in iter {
            let (_, value) = item?;
            let mut tx: PendingTx = bincode::deserialize(&value)?;

            if tx.status != TxStatus::Pending { continue; }

            // 策略1：有 hash → 直接查 receipt
            if let Some(hash) = tx.tx_hash {
                if let Ok(Some(receipt)) = self.provider.get_transaction_receipt(hash).await {
                    tx.status = if receipt.status == Some(1.into()) {
                        TxStatus::Confirmed
                    } else {
                        TxStatus::Failed
                    };
                    to_delete.push(tx.ts);
                }
            } else {
                // 策略2：无 hash（广播失败或掉单）→ 用 nonce + event 扫描兜底
                let current_nonce = self.provider.get_transaction_count(tx.from, None).await?;
                if tx.nonce < current_nonce {
                    // 说明这个 nonce 已经被其他交易占领 → 必然已上链或被顶掉
                    // 再扫一下从 tx.ts 时间点之后的所有 tx
                    let from_block = self.provider.get_block_number().await?.as_u64().saturating_sub(500); // 最近500块
                    let txs = self.provider.get_block_with_txs(BlockNumber::Number(from_block.into())).await?.unwrap().transactions;
                    
                    for onchain_tx in txs {
                        if onchain_tx.from == tx.from && onchain_tx.nonce == tx.nonce {
                            tx.tx_hash = Some(onchain_tx.hash);
                            tx.status = TxStatus::Confirmed;
                            to_delete.push(tx.ts);
                            break;
                        }
                        // 如果发现同一个 nonce 但 hash 不同 → 被 replace/dropped
                        if onchain_tx.from == tx.from && onchain_tx.nonce == tx.nonce && onchain_tx.hash != tx.tx_hash.unwrap_or_default() {
                            tx.status = TxStatus::Dropped;
                            to_delete.push(tx.ts);
                        }
                    }
                }
            }

            if matches!(tx.status, TxStatus::Confirmed | TxStatus::Dropped | TxStatus::Failed) {
                db.put(&tx.ts.to_be_bytes(), &bincode::serialize(&tx)?)?;
            }
        }

        // 确认成功/被顶掉的才真正删除（可选：也可以保留做历史）
        for ts in to_delete {
            db.delete(ts.to_be_bytes().to_vec())?;
        }

        Ok(())
    }

    pub async fn get_by_ts(&self, ts: i64) -> Option<PendingTx> {
        let db = self.db.read().await;
        db.get(ts.to_be_bytes().to_vec()).ok()??
            .map(|v| bincode::deserialize(&v).unwrap())
    }
}
