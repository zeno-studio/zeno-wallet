
use serde::{Deserialize, Serialize};
use alloy_primitives::{Address,address};
use crate::evm::assets::AssetsType;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum AssetsType {
    Currency,
    ERC20,
    ERC721,
    ERC1155,
    UNDEFINED,
}

pub trait IntoInterAsset {
    fn into_inter(self) -> AssetsType;
}
