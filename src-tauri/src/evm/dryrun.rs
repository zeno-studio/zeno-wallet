use revm::{
    primitives::{Address, U256, TxEnv, TransactTo, CfgEnv, BlockEnv},
    db::CacheDB,
    Context, Evm,
};

pub fn dry_run_eth_transfer(
    from: Address,
    to: Address,
    value: U256,
    block_env: BlockEnv,
) -> Result<u64, String> {
    let mut db = CacheDB::new(); // 空状态

    let tx = TxEnv {
        caller: from,
        transact_to: TransactTo::Call(to),
        value,
        data: vec![],
        gas_limit: 1_000_000,
        ..Default::default()
    };

    let mut evm = Evm::builder()
        .with_db(&mut db)
        .with_block_env(block_env)
        .with_cfg_env(CfgEnv::default())
        .with_tx_env(tx)
        .build();

    let result = evm.transact().map_err(|e| e.to_string())?;
    Ok(result.gas_used)
}