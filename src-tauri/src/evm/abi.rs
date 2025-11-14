use alloy_sol_types::{sol, SolStruct};

sol! {
    #[sol(rpc)]
    contract IERC20 {
        event Transfer(address indexed from, address indexed to, uint256 value);
        event Approval(address indexed owner, address indexed spender, uint256 value);
        function name() public view returns (string name);
        function symbol() public view returns (string symbol);
        function decimals() public view returns (uint8 decimals);
    }
}

sol! {
    #[sol(rpc)]
    contract IERC721 {
        event Transfer(address indexed from, address indexed to, uint256 indexed tokenId);
        event Approval(address indexed owner, address indexed approved, uint256 indexed tokenId);
        function name() public view returns (string name);
        function symbol() public view returns (string symbol);
    }

    #[sol(rpc)]
    contract IERC721WithMetadata is IERC721 {
        function tokenURI(uint256 tokenId) public view returns (string uri);
    }
}


// // Represent a Solidity type in rust
// type MySolType = FixedArray<Bool, 2>;

// let data = [true, false];
// let validate = true;

// // SolTypes expose their Solidity name :)
// assert_eq!(&MySolType::sol_type_name(), "bool[2]");

// // SolTypes are used to transform Rust into ABI blobs, and back.
// let encoded: Vec<u8> = MySolType::abi_encode(&data);
// let decoded: [bool; 2] = MySolType::abi_decode(&encoded)?;
// assert_eq!(data, decoded);

// // This is more easily done with the `SolValue` trait:
// let encoded: Vec<u8> = data.abi_encode();
// let decoded: [bool; 2] = <[bool; 2]>::abi_decode(&encoded)?;
// assert_eq!(data, decoded);
// # Ok::<_, alloy_sol_types::Error>(())