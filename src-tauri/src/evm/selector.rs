// src/interface.rs
// SPDX-License-Identifier: MIT
// 2025 年最精炼、最快的 EVM 接口选择器模块
// 使用 alloy_sol_types::sol! 编译时生成 4-byte selector

use alloy_sol_types::{sol, SolCall, SolInterface};

// ==================== 标准接口定义 + 自动 selector ====================

sol! {
    // ERC20 标准接口
    interface IERC20 {
        function totalSupply() external view returns (uint256);
        function balanceOf(address) external view returns (uint256);
        function transfer(address to, uint256 value) external returns (bool);
        function transferFrom(address from, address to, uint256 value) external returns (bool);
        function approve(address spender, uint256 value) external returns (bool);
        function allowance(address owner, address spender) external view returns (uint256);

        // EIP-2612
        function permit(address owner, address spender, uint256 value, uint256 deadline, uint8 v, bytes32 r, bytes32 s) external;
        function nonces(address owner) external view returns (uint256);
        function DOMAIN_SEPARATOR() external view returns (bytes32);
    }

    // ERC721 标准接口
    interface IERC721 {
        function balanceOf(address owner) external view returns (uint256);
        function ownerOf(uint256 tokenId) external view returns (address);
        function safeTransferFrom(address from, address to, uint256 tokenId) external;
        function safeTransferFrom(address from, address to, uint256 tokenId, bytes data) external;
        function transferFrom(address from, address to, uint256 tokenId) external;
        function approve(address to, uint256 tokenId) external;
        function getApproved(uint256 tokenId) external view returns (address);
        function setApprovalForAll(address operator, bool approved) external;
        function isApprovedForAll(address owner, address operator) external view returns (bool);

        // ERC721 Permit (EIP-4494)
        function permit(address spender, uint256 tokenId, uint256 deadline, uint8 v, bytes32 r, bytes32 s) external;
    }

    interface IERC165 {
        /// @dev Returns true if this contract implements the interface defined by `interfaceId`.
        /// See the corresponding EIP section to learn more about how these ids are created.
        /// https://eips.ethereum.org/EIPS/eip-165
        function supportsInterface(bytes4 interfaceId) external view returns (bool);
    }

    // ERC1155 标准接口
    interface IERC1155 {
        function balanceOf(address account, uint256 id) external view returns (uint256);
        function balanceOfBatch(address[] accounts, uint256[] ids) external view returns (uint256[] memory);
        function setApprovalForAll(address operator, bool approved) external;
        function isApprovedForAll(address account, address operator) external view returns (bool);
        function safeTransferFrom(address from, address to, uint256 id, uint256 amount, bytes data) external;
        function safeBatchTransferFrom(address from, address to, uint256[] ids, uint256[] amounts, bytes data) external;
    }

    // WETH9 / IWETH
    interface IWETH9 {
        function deposit() external payable;
        function withdraw(uint256 wad) external;
        function totalSupply() external view returns (uint256);
        function approve(address guy, uint256 wad) external returns (bool);
        function transfer(address dst, uint256 wad) external returns (bool);
        function transferFrom(address src, address dst, uint256 wad) external returns (bool);
    }

    // 常用 DeFi 额外接口（很多钱包都会查）
    interface ICommonDeFi {
        function swapExactTokensForTokens(uint256 amountIn, uint256 amountOutMin, address[] path, address to, uint256 deadline) external;
        function swapTokensForExactTokens(uint256 amountOut, uint256 amountInMax, address[] path, address to, uint256 deadline) external;
        function multicall(bytes[] data) external payable returns (bytes[] memory results);
        function multicall(uint256 deadline, bytes[] data) external payable returns (bytes[] memory results);
    }
}

// ==================== 通用选择器宏 + 函数 ====================

/// 编译时计算 selector，返回 [u8; 4]
/// 用法：selector!(IERC20::balanceOf)
#[macro_export]
macro_rules! selector {
    ($call:ty) => {{
        // alloy 在编译时自动生成 selector
        const SELECTOR: [u8; 4] = <$call as alloy_sol_types::SolCall>::SELECTOR;
        SELECTOR
    }};
}

/// 运行时从函数名字符串计算 selector（兼容 fallback）
pub const fn selector_str(name: &str) -> [u8; 4] {
    use alloy_primitives::keccak256;
    let signature = if name.contains('(') {
        name.as_bytes()
    } else {
        // 自动补全常见参数（仅限常见函数）
        match name {
            "balanceOf" => b"balanceOf(address)",
            "ownerOf" => b"ownerOf(uint256)",
            "approve" => b"approve(address,uint256)",
            "transfer" => b"transfer(address,uint256)",
            "transferFrom" => b"transferFrom(address,address,uint256)",
            "deposit" => b"deposit()",
            "withdraw" => b"withdraw(uint256)",
            "permit" => b"permit(address,address,uint256,uint256,uint8,bytes32,bytes32)",
            _ => name.as_bytes(),
        }
    };
    let hash = keccak256(signature);
    [hash[0], hash[1], hash[2], hash[3]]
}

// ==================== 常用选择器常量（推荐直接用宏）===================

pub mod selectors {
    use super::*;

    // ERC20
    pub const BALANCE_OF: [u8; 4] = selector!(IERC20::balanceOf);
    pub const ALLOWANCE: [u8; 4] = selector!(IERC20::allowance);
    pub const APPROVE: [u8; 4] = selector!(IERC20::approve);
    pub const TRANSFER: [u8; 4] = selector!(IERC20::transfer);
    pub const TRANSFER_FROM: [u8; 4] = selector!(IERC20::transferFrom);
    pub const PERMIT: [u8; 4] = selector!(IERC20::permit);

    // ERC721
    pub const OWNER_OF: [u8; 4] = selector!(IERC721::ownerOf);
    pub const GET_APPROVED: [u8; 4] = selector!(IERC721::getApproved);
    pub const IS_APPROVED_FOR_ALL: [u8; 4] = selector!(IERC721::isApprovedForAll);

    // ERC1155
    pub const BALANCE_OF_1155: [u8; 4] = selector!(IERC1155::balanceOf);
    pub const BALANCE_OF_BATCH: [u8; 4] = selector!(IERC1155::balanceOfBatch);

    // WETH
    pub const DEPOSIT: [u8; 4] = selector!(IWETH9::deposit);
    pub const WITHDRAW: [u8; 4] = selector!(IWETH9::withdraw);

    // DeFi
    pub const MULTICALL: [u8; 4] = selector!(ICommonDeFi::multicall);

      // ERC-165 
    pub const SUPPORTS_INTERFACE: [u8; 4] = selector!(IERC165::supportsInterface);
    // 结果是: [0x01, 0xff, 0xc9, 0xa7]
}

pub mod erc165 {
    use alloy_primitives::B256;

    pub const IERC165: [u8; 4]     = [0x01, 0xff, 0xc9, 0xa7];
    pub const IERC721: [u8; 4]     = [0x80, 0xac, 0x58, 0xcd];
    pub const IERC721_METADATA: [u8; 4] = [0x5b, 0x5e, 0x13, 0x9f];
    pub const IERC721_ENUMERABLE: [u8; 4] = [0x78, 0x0e, 0x9d, 0x63];
    pub const IERC1155: [u8; 4]    = [0xd9, 0xb6, 0x7a, 0x6e];
    pub const IERC1155_METADATA: [u8; 4] = [0x0e, 0x89, 0x36, 0x9c];
}

// ==================== 使用示例 ====================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_selectors() {
        assert_eq!(selectors::BALANCE_OF, [0x70, 0xa0, 0x82, 0x31]); // balanceOf(address)
        assert_eq!(selectors::APPROVE, [0x09, 0x5e, 0xa7, 0xb3]);    // approve(address,uint256)
        assert_eq!(selectors::PERMIT, [0xd5, 0x05, 0x25, 0x2f]);    // permit(...)
        assert_eq!(selectors::DEPOSIT, [0xd0, 0xe3, 0x0d, 0xb0]);    // deposit()

        // 运行时计算
        assert_eq!(selector_str("balanceOf"), [0x70, 0xa0, 0x82, 0x31]);
        assert_eq!(selector_str("balanceOf(address)"), [0x70, 0xa0, 0x82, 0x31]);
    }
}


// use crate::interface::{selector, selectors, selector_str};

// // 编译时（推荐，零成本）
// let sel = selector!(IERC20::balanceOf); // [u8; 4]

// // 常量使用
// if calldata[..4] == selectors::APPROVE { /* 高亮 approve */ }

// // 运行时（fallback）
// let sel = selector_str("balanceOf(address)");
// let sel = selector_str("permit"); // 自动补全


// if calldata[..4] == selectors::SUPPORTS_INTERFACE {
//     let interface_id = &calldata[4..8]; // 取 bytes4 参数
//     match interface_id {
//         erc165::IERC721 => println!("这是个 ERC721"),
//         erc165::IERC1155 => println!("这是个 ERC1155"),
//         erc165::IERC721_METADATA => println!("支持元数据"),
//         _ => {}
//     }
// }