// use revm::{
//     context::TxEnv,
//     database::{AlloyDB, CacheDB, EmptyDB},
//     primitives::{
//         address, hardfork::SpecId, keccak256, Address, StorageValue, TxKind, KECCAK_EMPTY, U256,
//     },
//     state::AccountInfo,
//     Context, Database, MainBuilder, MainContext,
// };
// use alloy_provider::{network::Ethereum, DynProvider, Provider, ProviderBuilder};
// use alloy::primitives::utils::{ parse_ether, format_ether };

// // Constants
// /// USDC token address on Ethereum mainnet
// pub const TOKEN: Address = address!("a0b86991c6218b36c1d19d4a2e9eb0ce3606eb48");
// /// Treasury address that receives ERC20 gas payments
// pub const TREASURY: Address = address!("0000000000000000000000000000000000000001");

// #[tokio::main]
// async fn main() -> Result<()> {
//     // Initialize the Alloy provider and database
//     let rpc_url = "https://mainnet.infura.io/v3/c60b0bb42f8a4c6481ecd229eddaca27";
//     let provider = ProviderBuilder::new().connect(rpc_url).await?.erased();
// }










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