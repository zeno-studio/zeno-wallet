
use serde::{Deserialize, Serialize};
use alloy_primitives::{Address,address};


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

pub fn mapper_assets_type(asset_type: &str) -> AssetsType {
    let asset_type = asset_type.to_lowercase();
    match asset_type {
        "Currency" => AssetsType::Currency,
        "erc20" => AssetsType::ERC20,
        "erc721" => AssetsType::ERC721,
        "erc1155" => AssetsType::ERC1155,
        _ => AssetsType::UNDEFINED,
    }
}
