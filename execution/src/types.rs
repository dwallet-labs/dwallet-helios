use std::{collections::HashMap, fmt};

use ethers::types::{Address, Bytes, H256, U256};
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone)]
pub struct Account {
    pub balance: U256,
    pub nonce: u64,
    pub code_hash: H256,
    pub code: Vec<u8>,
    pub storage_hash: H256,
    pub slots: HashMap<U256, U256>,
}

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CallOpts {
    pub from: Option<Address>,
    pub to: Option<Address>,
    pub gas: Option<U256>,
    pub gas_price: Option<U256>,
    pub value: Option<U256>,
    pub data: Option<Bytes>,
}

impl fmt::Debug for CallOpts {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CallOpts")
            .field("from", &self.from)
            .field("to", &self.to)
            .field("value", &self.value)
            .field("data", &hex::encode(self.data.clone().unwrap_or_default()))
            .finish()
    }
}

/// The `VerifyProofInput` struct is used to encapsulate the input parameters required for verifying
/// a Merkle proof in the context of Ethereum storage. It contains the proof path, the root hash,
/// the storage key hash, and the corresponding value.
/// # Fields
/// * `proof` - An array of RLP-serialized MerkleTree nodes. This array starts with the `storageHash`
///   node and follows the path of the SHA3 hash of the key. This represents the proof path from the
///   storage root to the desired storage value.
///
/// * `root` - A 32-byte vector representing the SHA3 hash of the StorageRoot. All storage data will
///   deliver a Merkle proof that starts with this root hash. This serves as the anchor for verifying
///   the proof.
///
/// * `path` - A vector of bytes representing the requested storage key hash. This is the specific key
///   within the storage trie for which the proof is being verified.
///
/// * `value` - A vector of bytes representing the storage value. This is the value stored at the location
///   identified by the key hash in the path field.
#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct ProofVerificationInput {
    // Array of rlp-serialized MerkleTree-Nodes, starting with the storageHash-Node,
    // following the path of the SHA3 (key) as a path.
    pub proof: Vec<Bytes>,
    //  32 Bytes - SHA3 of the StorageRoot.
    // All storage will deliver a MerkleProof starting with this rootHash.
    pub root: Vec<u8>,
    // The requested storage key hash.
    pub path: Vec<u8>,
    // The storage value.
    pub value: Vec<u8>,
}
