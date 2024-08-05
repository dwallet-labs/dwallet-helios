//! This module provides utilities for standardizing and calculating storage slots in Solidity
//! smart contracts. These utilities are essential for interacting with smart contract storage
//! in a predictable and reliable manner. The utilities include functions for calculating mapping
//! slots, and determining the storage slot for a given message and dWallet ID.
//! These functions follow the specifications and layout described in Solidity's documentation
//! for storage slots and mapping layouts.
//!
//! Solidity uses 256-bit hashes (H256) for storage slots and keys.
//! When converting integers or other types of data to fit this format, padding is required
//! to ensure they match the expected 256-bit length.
//!
//! The code in this module originates from the need to interact with smart contract storage
//! layouts programmatically, ensuring consistency and correctness in storage access patterns.
//! The implementation follows the specifications outlined in the
//! [Solidity documentation](https://docs.soliditylang.org/en/v0.8.24/internals/layout_in_storage.html#mappings-and-dynamic-arrays)

use ethers::{
    abi::ethabi::{encode, Token},
    types::H256,
};
use eyre::Error;
use sha3::{Digest, Keccak256};

/// Calculates the mapping slot for a given key and storage slot (in the contract's storage layout).
/// First initializes a new `Keccak256` hasher, then standardizes the input slot and key.
/// The standardized key and slot are then hashed together to produce a new `H256` hash.
/// The result hash will be used to get the location of the (key, value) pair in the contract's
/// storage.
/// # Arguments
/// * `key` – A H256 hash that represents the key for which the mapping slot is to be calculated.
///   The Key is `Keccak256(message + dwallet_id)`.
/// * `Mapping_slot` – A `u64` value that represents the mapping slot in the contract storage layout.
///   [For more info](https://docs.soliditylang.org/en/v0.8.24/internals/layout_in_storage.html#mappings-and-dynamic-arrays)
#[allow(unused)]
fn calculate_mapping_slot(key: H256, mapping_slot: u64) -> H256 {
    let slot_encoded = encode(&[Token::Uint(mapping_slot.into())]);
    let slot_encoded = H256::from_slice(&slot_encoded);

    let key_encoded = encode(&[Token::FixedBytes(key.as_bytes().to_vec())]);
    let key_encoded = H256::from_slice(&key_encoded);

    let mut hasher = Keccak256::new();
    hasher.update(key_encoded.as_bytes());
    hasher.update(slot_encoded.as_bytes());
    H256::from_slice(&hasher.finalize())
}

/// Calculates the key for a given message and dWallet ID.
/// In the smart contract, the key is calculated by hashing the message, and the dWallet ID
/// together. The result is a H256 hash that represents the key.
#[allow(unused)]
fn calculate_key(mut message: Vec<u8>, dwallet_id: Vec<u8>) -> H256 {
    let mut hasher = Keccak256::new();
    message.extend_from_slice(dwallet_id.as_slice());
    hasher.update(message);
    H256::from_slice(&hasher.finalize())
}

/// Calculates the storage slot for a given message, dWallet ID, and data slot.
/// The function first calculates a key by hashing the message, and the dWallet ID together.
/// Then, it calculates the mapping slot for the calculated key, and the provided data slot.
/// The calculated mapping slot can be used to locate the (key, value) pair in the contract's
/// storage. # Arguments
/// * `message` – A string that represents the message to be stored.
/// * `dwallet_id` – A vector of bytes that represents the dWallet ID.
/// * `data_slot` – An unsigned 64-bit integer that represents the data slot.
/// # Returns
/// * A `Result` that contains a 256-bit hash (H256) that represents the calculated storage slot, or
///   an `Error` if the calculation fails.
#[allow(unused)]
pub fn get_message_storage_slot(
    message: String,
    dwallet_id: Vec<u8>,
    data_slot: u64,
) -> Result<H256, Error> {
    // Calculate memory slot.
    // Each key is calculated by hashing the concatenation of the msg and dWalletID.
    let key = calculate_key(
        message.clone().as_bytes().to_vec(),
        dwallet_id.as_slice().to_vec(),
    );
    Ok(calculate_mapping_slot(key, data_slot))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calculate_mapping_slot_valid() {
        let key = H256::from_slice(&[0u8; 32]);
        let slot = 0;

        let expected_hash = {
            let mut hasher = Keccak256::new();
            hasher.update([0u8; 32]);
            hasher.update([0u8; 32]);
            H256::from_slice(&hasher.finalize())
        };

        assert_eq!(expected_hash, calculate_mapping_slot(key, slot));

        let key = H256::from_slice(&[1u8; 32]);
        let slot = u64::MAX;

        let expected_hash = {
            let mut hasher = Keccak256::new();
            hasher.update([1u8; 32]);
            hasher.update([
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 255,
                255, 255, 255, 255, 255, 255,
            ]);
            H256::from_slice(&hasher.finalize())
        };

        assert_eq!(expected_hash, calculate_mapping_slot(key, slot));
    }

    #[test]
    fn calculate_key_valid() {
        let message: Vec<u8> = vec![];
        let dwallet_id: Vec<u8> = vec![];
        let expected_hash = {
            let mut hasher = Keccak256::new();
            hasher.update([]);
            H256::from_slice(&hasher.finalize())
        };

        assert_eq!(expected_hash, calculate_key(message, dwallet_id));

        let dwallet_id = "be344ddffaa7a8c9c5ae7f2d09a77f20ed54f93bf5e567659feca5c3422ae7a6";
        let byte_vec_dwallet_id = hex::decode(dwallet_id).expect("Invalid hex string");
        let message = [1u8; 32].to_vec();

        let expected_hash = {
            let mut hasher = Keccak256::new();
            let mut combined = message.clone();
            combined.extend_from_slice(&byte_vec_dwallet_id);
            hasher.update(combined);
            H256::from_slice(&hasher.finalize())
        };

        assert_eq!(expected_hash, calculate_key(message, byte_vec_dwallet_id))
    }

    #[test]
    fn get_message_storage_slot_valid() {
        let message = "test_message".to_string();
        let dwallet_id = "be344ddffaa7a8c9c5ae7f2d09a77f20ed54f93bf5e567659feca5c3422ae7a6";
        let byte_vec_dwallet_id = hex::decode(dwallet_id).expect("Invalid hex string");
        let data_slot = 12345u64;

        let key = calculate_key(
            message.clone().as_bytes().to_vec(),
            byte_vec_dwallet_id.clone(),
        );
        let expected_slot = calculate_mapping_slot(key, data_slot);

        let result = get_message_storage_slot(message, byte_vec_dwallet_id, data_slot).unwrap();

        assert_eq!(expected_slot, result);
    }
}
