//! This module provides functionality for creating Merkle proofs related to Ethereum account
//! and storage states.

use ethers::{prelude::EIP1186ProofResponse, utils::rlp::RlpStream};

/// Encodes an Ethereum account using RLP encoding according to the Ethereum specifications.
/// These four elements are the fundamental components of an Ethereum account,
/// and thus they are the ones that need to be encoded to represent the account’s state fully.
/// You can read more about accounts' state
/// in [Ethereum Yellow Paper](https://ethereum.github.io/yellowpaper/paper.pdf), section 4.1.
/// More info about RLP encoding [here](https://ethereum.org/en/developers/docs/data-structures-and-encoding/rlp/).
pub fn encode_account(proof: &EIP1186ProofResponse) -> Vec<u8> {
    let mut stream = RlpStream::new_list(4);
    stream.append(&proof.nonce);
    stream.append(&proof.balance);
    stream.append(&proof.storage_hash);
    stream.append(&proof.code_hash);
    let encoded = stream.out();
    encoded.to_vec()
}
