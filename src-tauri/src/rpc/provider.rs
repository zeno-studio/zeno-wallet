use once_cell::sync::Lazy;
use std::collections::HashMap;

pub static PUBLIC_RPC_ENDPOINTS: Lazy<HashMap<&'static str, Vec<&'static str>>> = Lazy::new(|| {
    HashMap::from([
        ("eth", vec![
            "https://rpc.ankr.com/eth",
            "https://eth-mainnet.public.blastapi.io",
            "https://ethereum.publicnode.com",
        ]),
        ("bsc", vec![
            "https://bsc-dataseed.binance.org",
            "https://rpc.ankr.com/bsc",
            "https://bsc.publicnode.com",
        ]),
        ("polygon", vec![
            "https://polygon-rpc.com",
            "https://rpc.ankr.com/polygon",
            "https://polygon-bor.publicnode.com",
        ]),
    ])
});




