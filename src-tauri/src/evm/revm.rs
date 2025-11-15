// use revm::primitives::{BlockEnv, TxEnv, TransactTo, U256, Address};
// use revm::{db::CacheDB, Evm, CfgEnv};
// use helios::client::ClientBuilder;
// use helios::config::networks::Mainnet;

// pub async fn dry_run_with_helios(
//     client: &helios::client::Client,
//     from: Address,
//     to: Address,
//     value: U256,
// ) -> Result<u64, String> {
//     // 从 Helios 获取 BlockEnv（零 RPC）
//     let header = client.get_execution_header_by_number(None).await
//         .map_err(|e| e.to_string())?;
//     let block_env = BlockEnv {
//         number: header.number.into(),
//         timestamp: header.timestamp.into(),
//         basefee: header.base_fee_per_gas.map(Into::into).unwrap_or_default(),
//         gas_limit: header.gas_limit.into(),
//         ..Default::default()
//     };

//     // revm dry-run
//     let mut db = CacheDB::new();
//     let tx = TxEnv {
//         caller: from,
//         transact_to: TransactTo::Call(to),
//         value,
//         data: vec![],
//         gas_limit: 1_000_000,
//         ..Default::default()
//     };
//     let mut evm = Evm::builder()
//         .with_db(&mut db)
//         .with_block_env(block_env)
//         .with_cfg_env(CfgEnv::default())
//         .with_tx_env(tx)
//         .build();
//     let result = evm.transact().map_err(|e| e.to_string())?;

//     Ok(result.gas_used)
// }

// // 使用
// // let client = ClientBuilder::new().network(Mainnet).build()?;
// // let gas = dry_run_with_helios(&client, from, to, value).await?;