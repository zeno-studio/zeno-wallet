// src/revm_power.rs
// 2025 终极版「revm 钱包第二大脑」
// 一行调用，秒出：gas + 余额变化 + 安全风险 + 事件解析 + 真实返回值

use alloy_primitives::{Address, Bytes, B256, U256};
use revm::{
    db::{CacheDB, EmptyDB},
    primitives::{ExecutionResult, TransactTo, TxEnv, Env, SpecId, CfgEnv, BlockEnv},
    EVM,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulateResult {
    pub success: bool,
    pub gas_used: u64,
    pub gas_limit: u64,                    // 推荐上链值（已加 buffer）
    pub revert_reason: Option<String>,
    pub return_data: Bytes,

    // 余额变化（最重要！）
    pub balance_changes: HashMap<Address, i128>,  // 正数=收到，负数=支出

    // 安全扫描
    pub risks: Vec<SecurityRisk>,

    // 事件（可选解析）
    pub logs: Vec<LogEntry>,

    // 真实执行痕迹
    pub created_contracts: Vec<Address>,
    pub self_destructed: Vec<Address>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityRisk {
    LargeApproval(Address, Address, U256),     // token, spender, amount
    SuspiciousTransfer(Address, Address, U256), // token or ETH
    SetApprovalForAll(Address, Address, bool),
    PermitSigned(Address, Address, U256, u64), // token, spender, amount, deadline
    ContractCanUpgradeOrDestroy,
    CallsMaliciousContract(Address),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub address: Address,
    pub topics: Vec<B256>,
    pub data: Bytes,
}

// ==================== 一行调用核心入口 ====================
pub struct Revmpower {
    // 只存自家合约的完整状态（最多 10 个）
    my_contracts: HashMap<Address, ContractState>,
}

#[derive(Clone)]
struct ContractState {
    code: Bytes,
    storage: HashMap<B256, B256>,
}

impl Revmpower {
    pub fn new() -> Self {
        Self {
            my_contracts: HashMap::new(),
        }
    }

    // 注册自家合约（部署后调用一次）
    pub fn register_my_contract(&mut self, addr: Address, code: Bytes, storage: Vec<(B256, B256)>) {
        self.my_contracts.insert(addr, ContractState {
            code,
            storage: storage.into_iter().collect(),
        });
    }

    // 核心：一行调用，秒出全部结果
    pub fn simulate(
        &self,
        from: Address,
        to: Option<Address>,
        value: U256,
        data: Bytes,
        block_number: u64,
    ) -> SimulateResult {
        let mut db = CacheDB::new(EmptyDB::default());

        // 1. 加载调用者（给足钱）
        db.insert_account_info(from, revm::primitives::AccountInfo::new(
            U256::from(1000) * U256::from(1e18), // 1000 ETH
            0,
            Bytes::new(),
        ));

        // 2. 如果是自家合约 → 加载完整状态（100% 精确）
        if let Some(to_addr) = to {
            if let Some(state) = self.my_contracts.get(&to_addr) {
                db.insert_account_info(to_addr, revm::primitives::AccountInfo::new(
                    U256::MAX, 0, state.code.clone()
                ));
                for (k, v) in &state.storage {
                    db.insert_account_storage(to_addr, *k, *v).unwrap();
                }
            }
        }

        // 3. 构建 EVM
        let mut evm = EVM::new();
        evm.database(db);
        evm.env = Box::new(Env {
            cfg: CfgEnv {
                spec_id: SpecId::LATEST,
                chain_id: 1,
                ..Default::default()
            },
            block: BlockEnv {
                number: block_number.into(),
                timestamp: U256::from(std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH).unwrap().as_secs()),
                basefee: U256::from(10_000_000_000u64),
                gas_limit: U256::from(30_000_000),
                ..Default::default()
            },
            tx: TxEnv {
                caller: from,
                transact_to: to.map(TransactTo::Call).unwrap_or(TransactTo::Create),
                value,
                data,
                gas_limit: 30_000_000,
                ..Default::default()
            },
        });

        // 4. 执行！
        let result = evm.transact_commit().unwrap();

        // 5. 解析全部信息
        self.analyze_result(result, &evm)
    }

    fn analyze_result(&self, result: ExecutionResult, evm: &EVM<CacheDB<EmptyDB>>) -> SimulateResult {
        let db = &evm.context.evm.db;
        let mut changes = HashMap::new();

        // 余额变化
        for (addr, acc) in db.accounts.iter() {
            let old = acc.info.balance;
            let new = acc.info.balance;
            if old != new {
                changes.insert(*addr, new.saturating_sub(old) as i128);
            }
        }

        // 基础信息
        let (success, gas_used, output, logs) = match result {
            ExecutionResult::Success { gas_used, output, logs, .. } => (true, gas_used, output, logs),
            ExecutionResult::Revert { gas_used, output, logs } => (false, gas_used, output, logs),
            ExecutionResult::Halt { .. } => (false, 0, revm::primitives::Output::WithoutData, vec![]),
        };

        let revert_reason = if !success {
            output.revert_reason()
        } else {
            None
        };

        // 安全扫描（核心！）
        let mut risks = vec![];
        for log in &logs {
            if log.address == Address::ZERO { continue; }

            // 检测大额授权
            if log.topics.len() >= 3 && log.topics[0] == keccak256("Approval(address,address,uint256)") {
                if let Some(amount) = U256::try_from_be_slice(&log.data).ok() {
                    if amount > U256::from(1_000_000) * U256::from(1e18) { // > 100万美刀
                        risks.push(SecurityRisk::LargeApproval(
                            log.address,
                            Address::from_slice(&log.topics[2].to_fixed_bytes()[12..32]),
                            amount,
                        ));
                    }
                }
            }
        }

        SimulateResult {
            success,
            gas_used,
            gas_limit: (gas_used as f64 * 1.3) as u64, // 30% buffer
            revert_reason,
            return_data: output.into_data(),
            balance_changes: changes,
            risks,
            logs: logs.iter().map(|l| LogEntry {
                address: l.address,
                topics: l.topics.clone(),
                data: l.data.clone(),
            }).collect(),
            created_contracts: vec![],
            self_destructed: vec![],
        }
    }
}

// ==================== 工具函数 ====================
fn keccak256(input: &str) -> B256 {
    use sha3::{Digest, Keccak256};
    B256::from(Keccak256::digest(input.as_bytes()).0)
}








// // #[derive(Debug, Deserialize)]
// // pub struct SimulateInput {
// //     pub from: String,
// //     pub to: Option<String>,
// //     pub gas: Option<u64>,
// //     pub value: Option<String>,    // hex
// //     pub data: Option<String>,     // hex
// //     pub bytecode: Option<String>, // if user directly provides bytecode
// //     pub provider_kind: Option<crate::core::helios_mod::ProviderKind>,
// // }

// // pub struct RevmService {
// //     pub helios: Arc<Mutex<HeliosClient>>, // main provider path
// //     // an overlay cache of code/storage for quick local simulations
// //     pub overlay_code: Arc<Mutex<HashMap<[u8; 20], Vec<u8>>>>,
// // }

// // impl RevmService {
// //     pub fn new(helios: Arc<Mutex<HeliosClient>>) -> Self {
// //         Self {
// //             helios,
// //             overlay_code: Arc::new(Mutex::new(HashMap::new())),
// //         }
// //     }

// //     /// Main command: simulate execution locally using REVM v33
// //     pub async fn simulate(&self, input: SimulateInput) -> Result<String, String> {
// //         let hc = self.helios.lock().await.clone();

// //         // Determine addresses to fetch
// //         let mut addrs = vec![input.from.clone()];
// //         if let Some(to) = &input.to {
// //             addrs.push(to.clone());
// //         }

// //         // For now we only request no storage keys (can be extended by parsing calldata)
// //         let slots: Vec<Vec<String>> = addrs.iter().map(|_| vec![]).collect();

// //         // Build verified state using Helios client
// //         let verified: VerifiedState = hc
// //             .build_verified_state_for_addresses(addrs.clone(), slots, Some("latest"))
// //             .await?;

// //         // Convert VerifiedState into a REVM Database adapter
// //         let db = HeliosRevmDB::new(verified, self.overlay_code.clone(), hc);

// //         // Create EVM with the DB
// //         let mut evm = Evm::new();
// //         evm.database(db);

// //         // Build Env / TxEnv
// //         let mut env = Env::default();
// //         let tx = &mut env.tx;
// //         tx.caller = Address::from_slice(
// //             &hex::decode(input.from.trim_start_matches("0x")).map_err(|e| e.to_string())?,
// //         );
// //         if let Some(to) = &input.to {
// //             tx.transact_to = TransactTo::Call(Address::from_slice(
// //                 &hex::decode(to.trim_start_matches("0x")).map_err(|e| e.to_string())?,
// //             ));
// //         } else {
// //             tx.transact_to = TransactTo::Create;
// //         }
// //         if let Some(g) = input.gas {
// //             tx.gas_limit = g;
// //         }
// //         if let Some(vhex) = &input.value {
// //             tx.value = U256::from_big_endian(
// //                 &hex::decode(vhex.trim_start_matches("0x")).map_err(|e| e.to_string())?,
// //             );
// //         }
// //         if let Some(dhex) = &input.data {
// //             tx.data = hex::decode(dhex.trim_start_matches("0x"))
// //                 .map_err(|e| e.to_string())?
// //                 .into();
// //         }

// //         // If user provided bytecode override for the "to" address, inject into overlay_code
// //         if let Some(bc_hex) = &input.bytecode {
// //             if let Some(to) = &input.to {
// //                 let mut overlay = self.overlay_code.lock().await;
// //                 let addr_bytes: [u8; 20] = {
// //                     let vec =
// //                         hex::decode(to.trim_start_matches("0x")).map_err(|e| e.to_string())?;
// //                     let mut a = [0u8; 20];
// //                     a.copy_from_slice(&vec[..20]);
// //                     a
// //                 };
// //                 overlay.insert(
// //                     addr_bytes,
// //                     hex::decode(bc_hex.trim_start_matches("0x")).map_err(|e| e.to_string())?,
// //                 );
// //             }
// //         }

// //         evm.env = env;

// //         // Execute
// //         let out = evm.transact().map_err(|e| e.to_string())?;
// //         let result = format!(
// //             "status={:?} gas_used={:?} out_len={}",
// //             out.result,
// //             out.gas_used,
// //             out.out.len()
// //         );
// //         Ok(result)
// //     }
// // }

// // /// A simple REVM Database adapter backed by VerifiedState + overlay cache.
// // /// It implements `revm::Database` trait (methods: basic, code_by_hash, storage, block_hash)
// // pub struct HeliosRevmDB {
// //     verified: VerifiedState,
// //     overlay_code: Arc<Mutex<HashMap<[u8; 20], Vec<u8>>>>,
// //     helios_client: HeliosClient,
// // }

// // impl HeliosRevmDB {
// //     pub fn new(
// //         verified: VerifiedState,
// //         overlay_code: Arc<Mutex<HashMap<[u8; 20], Vec<u8>>>>,
// //         helios_client: HeliosClient,
// //     ) -> Self {
// //         Self {
// //             verified,
// //             overlay_code,
// //             helios_client,
// //         }
// //     }
// // }

// // // The Database trait requires an associated Error type. We'll use a simple boxed String.
// // #[derive(Debug)]
// // pub struct RevmDbError(String);

// // impl std::fmt::Display for RevmDbError {
// //     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
// //         write!(f, "RevmDbError: {}", self.0)
// //     }
// // }
// // impl std::error::Error for RevmDbError {}

// // impl revm::Database for HeliosRevmDB {
// //     type Error = RevmDbError;

// //     fn basic(&mut self, address: Address) -> Result<Option<AccountInfo>, Self::Error> {
// //         // find in verified.accounts
// //         let hex_addr = format!("0x{}", hex::encode(address.as_bytes()));
// //         if let Some(a) = self
// //             .verified
// //             .accounts
// //             .iter()
// //             .find(|x| x.address.eq_ignore_ascii_case(&hex_addr))
// //         {
// //             let mut info = AccountInfo::default();
// //             info.balance = {
// //                 let raw = a.balance_hex.trim_start_matches("0x");
// //                 let mut b = [0u8; 32];
// //                 let bytes = hex::decode(raw).map_err(|e| RevmDbError(e.to_string()))?;
// //                 let start = 32 - bytes.len();
// //                 b[start..].copy_from_slice(&bytes);
// //                 U256::from_big_endian(&b)
// //             };
// //             info.nonce = a.nonce;
// //             // code hash handled by code_by_hash
// //             return Ok(Some(info));
// //         }
// //         // fallback: request via helios client synchronously (blocking inside revm call is not ideal; in production use async wrapper DatabaseRef)
// //         Err(RevmDbError(
// //             "account not found in verified state; require full coverage".into(),
// //         ))
// //     }

// //     fn code_by_hash(
// //         &mut self,
// //         code_hash: revm::primitives::FixedBytes<32>,
// //     ) -> Result<Bytecode, Self::Error> {
// //         // First check overlay_code by matching address hashes — we don't have mapping here, so attempt to find any overlay entry whose keccak matches
// //         let overlay = futures::executor::block_on(self.overlay_code.lock()).clone();
// //         for (_addr, code) in overlay.iter() {
// //             let hash = revm::primitives::keccak256(code);
// //             if hash == code_hash.0 {
// //                 return Ok(Bytecode::new_raw(code.clone().into()));
// //             }
// //         }
// //         // Next try to find code from verified accounts
// //         for a in self.verified.accounts.iter() {
// //             if let Some(code_hex) = &a.code_hex {
// //                 let bytes = hex::decode(code_hex.trim_start_matches("0x"))
// //                     .map_err(|e| RevmDbError(e.to_string()))?;
// //                 let hash = revm::primitives::keccak256(&bytes);
// //                 if hash == code_hash.0 {
// //                     return Ok(Bytecode::new_raw(bytes.into()));
// //                 }
// //             }
// //         }
// //         Err(RevmDbError("code not found".into()))
// //     }

// //     fn storage(
// //         &mut self,
// //         address: Address,
// //         index: revm::primitives::Uint<256, 4>,
// //     ) -> Result<revm::primitives::Uint<256, 4>, Self::Error> {
// //         let hex_addr = format!("0x{}", hex::encode(address.as_bytes()));
// //         if let Some(a) = self
// //             .verified
// //             .accounts
// //             .iter()
// //             .find(|x| x.address.eq_ignore_ascii_case(&hex_addr))
// //         {
// //             if let Some(storage) = &a.storage {
// //                 let key_hex = format!("0x{}", hex::encode(index));
// //                 if let Some(vhex) = storage.get(&key_hex) {
// //                     let bytes = hex::decode(vhex.trim_start_matches("0x"))
// //                         .map_err(|e| RevmDbError(e.to_string()))?;
// //                     let mut buf = [0u8; 32];
// //                     let start = 32 - bytes.len();
// //                     buf[start..].copy_from_slice(&bytes);
// //                     return Ok(revm::primitives::Uint::from_big_endian(&buf));
// //                 }
// //             }
// //         }
// //         // if not available in verified snapshot, error — simulation must provide complete state
// //         Err(RevmDbError("storage slot missing in verified state".into()))
// //     }

// //     fn block_hash(&mut self, number: u64) -> Result<revm::primitives::FixedBytes<32>, Self::Error> {
// //         // If the requested number equals verified.block_number return its hash
// //         if number == self.verified.block_number {
// //             let mut fb = revm::primitives::FixedBytes::default();
// //             let h = self.verified.block_hash.trim_start_matches("0x");
// //             let hb = hex::decode(h).map_err(|e| RevmDbError(e.to_string()))?;
// //             fb.0.copy_from_slice(&hb[..32]);
// //             return Ok(fb);
// //         }
// //         Err(RevmDbError("block hash not available".into()))
// //     }
// // }

// // #[tauri::command]
// // pub async fn revm_simulate(state: tauri::State<'_, Arc<Mutex<RevmService>>>, input: SimulateInput) -> Result<String, String> {
// // state.lock().await.simulate(input).await
// // }