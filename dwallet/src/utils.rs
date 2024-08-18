//! This module provides functionality for creating Merkle proofs related to Ethereum account
//! and storage states.

use consensus::types::Bytes32;
use ethers::{
    prelude::{Address, EIP1186ProofResponse, H256, U256},
    utils::{keccak256, rlp::RlpStream},
};
use execution::types::ProofVerificationInput;
use eyre::{eyre, Report};

/// `1u8` for `True`.
/// If a message is approved, the value in the contract's storage map would be
/// `True`.
/// (Conversion from `True` to `1u8` happens on contract's side).
const TRUE_VALUE: u8 = 1;

/// Creates a proof verification input for an Ethereum account.
pub fn create_account_proof(
    contract_addr: &Address,
    state_root: &Bytes32,
    proof: &EIP1186ProofResponse,
) -> ProofVerificationInput {
    let account_path = keccak256(contract_addr.as_bytes()).to_vec();
    let account_encoded = encode_account(proof);

    ProofVerificationInput {
        proof: proof.clone().account_proof,
        root: state_root.as_slice().to_vec(),
        path: account_path,
        value: account_encoded,
    }
}

/// Encodes an Ethereum account using RLP encoding according to the Ethereum specifications.
/// These four elements are the fundamental components of an Ethereum account,
/// and thus they are the ones that need to be encoded to represent the account’s state fully.
/// You can read more about accounts' state
/// in [Ethereum Yellow Paper](https://ethereum.github.io/yellowpaper/paper.pdf), section 4.1.
/// More info about RLP encoding [here](https://ethereum.org/en/developers/docs/data-structures-and-encoding/rlp/).
pub(crate) fn encode_account(proof: &EIP1186ProofResponse) -> Vec<u8> {
    let mut stream = RlpStream::new_list(4);
    stream.append(&proof.nonce);
    stream.append(&proof.balance);
    stream.append(&proof.storage_hash);
    stream.append(&proof.code_hash);
    let encoded = stream.out();
    encoded.to_vec()
}

/// Extracts the storage proof for a specific message from the proof response
/// and returns a [`ProofVerificationInput`] with the storage proof.
pub fn extract_storage_proof(
    message_map_index: H256,
    proof: EIP1186ProofResponse,
) -> Result<ProofVerificationInput, Report> {
    // The storage proof for the specific message and dWalletID in the mapping.
    let msg_storage_proof = proof
        .storage_proof
        .iter()
        .find(|p| p.key == U256::from(message_map_index.as_bytes()))
        .ok_or_else(|| eyre!("Storage proof not found"))?;

    let storage_value = [TRUE_VALUE].to_vec();
    let mut msg_storage_proof_key_bytes = [0u8; 32];
    msg_storage_proof
        .key
        .to_big_endian(&mut msg_storage_proof_key_bytes);
    let storage_key_hash = keccak256(msg_storage_proof_key_bytes);

    Ok(ProofVerificationInput {
        proof: msg_storage_proof.clone().proof,
        root: proof.storage_hash.as_bytes().to_vec(),
        path: storage_key_hash.to_vec(),
        value: storage_value,
    })
}
