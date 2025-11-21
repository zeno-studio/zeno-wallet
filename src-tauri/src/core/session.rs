

pub struct SessionConfig {
    pub current_account: Option<Account>,
    pub current_chain: Option<Chain>,
    pub is_screen_locked: Option<bool>,
    pub is_wallet_locked: Option<bool>,
    pub hide_balance: Option<bool>,   
    pub active_host: Option<String>,        // 当前连接的 dApp（用于防钓鱼提示）
    pub pending_sign_request: Option<SignRequest>, // 当前待签名的交易/消息（防多开弹窗）
    pub hot_tokens: Option<Vec<String>>,
    pub hot_nfts: Option<Vec<String>>,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            current_account_index: Some(0),
            current_chain: None,
            is_screen_locked: Some(false),
            is_wallet_locked: Some(false),
        }
    }
}