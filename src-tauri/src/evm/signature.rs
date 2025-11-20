use alloy_primitives::{Address,U256, B256, Signature, keccak256};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;

pub(crate) fn verify_signature(hash: &B256, sig: &Signature, expected_addr: Address) -> bool {
    sig.recover_address_from_prehash(hash)
        .map_or(false, |recovered| recovered == expected_addr)
}

