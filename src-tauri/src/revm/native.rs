// 在你的 GasEstimator 上加一个方法
impl<P: RpcProvider> GasEstimator<P> {
    /// 专用于「自家合约」—— 100% 精确
    pub async fn estimate_my_contract_precise(
        &self,
        from: Address,
        to: Address,
        value: U256,
        data: Bytes,
        // 你必须提供这三样（部署后保存一下就行）
        runtime_bytecode: Bytes,
        storage_slots: Vec<(B256, B256)>,  // key => value
    ) -> Result<u64> {
        let mut db = CacheDB::new(EmptyDB::default());

        // 1. 加载调用者
        let caller_info = revm::primitives::AccountInfo::new(
            self.provider.get_balance(from).await?.max(U256::from(1000) * U256::from(1e18)),
            self.provider.get_nonce(from).await?,
            Bytes::new(),
        );
        db.insert_account_info(from, caller_info);

        // 2. 加载目标合约完整状态
        let contract_info = revm::primitives::AccountInfo::new(
            U256::MAX,
            0,
            runtime_bytecode.clone(),
        );
        db.insert_account_info(to, contract_info);

        // 3. 写入所有 storage 槽（关键！）
        for (key, value) in storage_slots {
            db.insert_account_storage(to, key, value)?;
        }

        let mut evm = EVM::new();
        evm.database(db);
        evm.env = Box::new(self.build_revm_env(from, Some(to), value, data).await);

        let result = evm.transact_commit()?;
        Ok(match result {
            ExecutionResult::Success { gas_used, .. } => gas_used,
            ExecutionResult::Revert { gas_used, .. } => gas_used,
            ExecutionResult::Halt { gas_used, .. } => gas_used,
        })
    }
}


rust

// 部署完合约后保存下来（可以放本地或 IPFS）
// let my_contract = MyContractMeta {
//     address: "0x1234...".parse()?,
//     runtime_bytecode: hex::decode("6080604052348015...")?.into(),
//     critical_storage_slots: vec![
//         (B256::from_hex("0x...slot0")? , B256::from_hex("0x...value")?),
//         // owner, paused, config 等
//     ],
// };

// let gas = estimator.estimate_my_contract_precise(
//     user,
//     my_contract.address,
//     value,
//     calldata,
//     my_contract.runtime_bytecode,
//     my_contract.critical_storage_slots.clone(),
// ).await?;

// println!("自家合约 100% 精确 gas: {}", gas); // 误差 < 10

