
use crate::core::state::Account;
use crate::data::nft::Nft;
use crate::evm::chain::Chain;
use crate::evm::token::Token;
use crate::data::tx::PendingTx;
 

pub struct SessionConfig {
    pub current_account: Option<Account>,
    pub current_chain: Option<Chain>,
    pub helios_current_chain: Option<Chain>,
    pub is_screen_locked: Option<bool>,
    pub is_wallet_locked: Option<bool>,
    pub hide_balance: Option<bool>,   
    pub active_dapp_host: Option<String>,        // 当前连接的 dApp（用于防钓鱼提示）
    pub pending_tx: Option<PendingTx>, // 当前待签名的交易/消息（防多开弹窗）
    pub user_tokens: Option<Vec<Token>>,
    pub user_nfts: Option<Vec<Nft>>,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            current_account_index: None,
            current_chain: None,
            helios_current_chain: None,
            hide_balance: Some(false),
            active_dapp_host: None,
            pending_tx: None,
            user_tokens: None,
            user_nfts: None,
            is_screen_locked: Some(false),
            is_wallet_locked: Some(false),
        }
    }
}


