//场景 1：读操作 → Multicall3（无需签名）

let (total_supply, balance) = provider
    .multicall()
    .add_call(weth.totalSupply())
    .add_call(weth.balanceOf(alice))
    .aggregate3()  // ← 读调用，自动批处理
    .await?;

println!("Supply: {total_supply}, Balance: {balance}");


