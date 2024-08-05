//! This module provides functionality for creating Merkle proofs related to Ethereum account
//! and storage states.

use consensus::types::Bytes32;
use ethers::{
    prelude::{Address, EIP1186ProofResponse, H256, U256},
    utils::{keccak256, rlp::RlpStream},
};
use execution::types::ProofVerificationInput;
use eyre::{eyre, Report};

/// Creates a proof verification input for an Ethereum account.
pub(crate) fn create_account_proof(
    contract_addr: &Address,
    state_root: &Bytes32,
    proof: &EIP1186ProofResponse,
) -> ProofVerificationInput {
    let account_path = keccak256(contract_addr.as_bytes()).to_vec();
    let account_encoded = encode_account(proof);

    let account_proof = ProofVerificationInput {
        proof: proof.clone().account_proof,
        root: state_root.as_slice().to_vec(),
        path: account_path,
        value: account_encoded,
    };
    account_proof
}

/// Encodes an Ethereum account using RLP encoding according to the Ethereum specifications.
/// More info [here](https://ethereum.org/en/developers/docs/data-structures-and-encoding/rlp/).
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
/// and returns a ProofVerificationInput with the storage proof.
pub(crate) fn extract_storage_proof(
    message_map_index: H256,
    proof: EIP1186ProofResponse,
) -> Result<ProofVerificationInput, Report> {
    // The storage proof for the specific message and dWalletID in the mapping.
    let msg_storage_proof = proof
        .storage_proof
        .iter()
        .find(|p| p.key == U256::from(message_map_index.as_bytes()))
        .ok_or_else(|| eyre!("Storage proof not found"))?;

    // 1 for True (if the message is approved, the value in the contract's storage map would be
    // True).
    let storage_value = [1].to_vec();
    let mut msg_storage_proof_key_bytes = [0u8; 32];
    msg_storage_proof
        .key
        .to_big_endian(&mut msg_storage_proof_key_bytes);
    let storage_key_hash = keccak256(msg_storage_proof_key_bytes);

    let storage_proof = ProofVerificationInput {
        proof: msg_storage_proof.clone().proof,
        root: proof.storage_hash.as_bytes().to_vec(),
        path: storage_key_hash.to_vec(),
        value: storage_value,
    };
    Ok(storage_proof)
}
