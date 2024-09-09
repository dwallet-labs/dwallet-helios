use std::{fmt::Display, str::FromStr};

use eyre::Result;
use ssz_rs::prelude::*;
use superstruct::superstruct;

use self::{
    primitives::{ByteList, ByteVector, U64},
    utils::{header_deserialize, superstruct_ssz, u256_deserialize},
};
pub mod primitives;
pub(crate) mod utils;

pub type Address = ByteVector<20>;
pub type Bytes32 = ByteVector<32>;
pub type LogsBloom = ByteVector<256>;
pub type BLSPubKey = ByteVector<48>;
pub type SignatureBytes = ByteVector<96>;
pub type Transaction = ByteList<1073741824>;

#[derive(serde::Deserialize, serde::Serialize, Debug, Default, SimpleSerialize, Clone)]
pub struct BeaconBlock {
    pub slot: U64,
    pub proposer_index: U64,
    pub parent_root: Bytes32,
    pub state_root: Bytes32,
    #[serde(skip_serializing, default)]
    pub body: BeaconBlockBody,
}

/// Represents the different types of Beacon blocks.
pub enum BeaconBlockType {
    Bellatrix,
    Capella,
    Deneb,
}

impl Display for BeaconBlockType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            BeaconBlockType::Bellatrix => "bellatrix".to_string(),
            BeaconBlockType::Capella => "capella".to_string(),
            BeaconBlockType::Deneb => "deneb".to_string(),
        };
        write!(f, "{}", str)
    }
}

impl FromStr for BeaconBlockType {
    type Err = ();

    /// Converts a string to a [`BeaconBlockType`].
    fn from_str(s: &str) -> std::result::Result<BeaconBlockType, ()> {
        match s {
            "bellatrix" => Ok(BeaconBlockType::Bellatrix),
            "capella" => Ok(BeaconBlockType::Capella),
            "deneb" => Ok(BeaconBlockType::Deneb),
            _ => Err(()),
        }
    }
}

/// Represents the different types of Beacon block bodies.
pub enum BeaconBlockBodyWrapper {
    Bellatrix(BeaconBlockBodyBellatrix),
    Capella(BeaconBlockBodyCapella),
    Deneb(BeaconBlockBodyDeneb),
}

impl BeaconBlockBodyWrapper {
    /// Creates a new [`BeaconBlockBodyWrapper`] from a JSON string, given the block type.
    pub fn new_from_json(json: Vec<u8>, block_type: &BeaconBlockType) -> Result<Self> {
        match block_type {
            BeaconBlockType::Bellatrix => {
                Ok(BeaconBlockBodyWrapper::Bellatrix(serde_json::from_slice::<
                    BeaconBlockBodyBellatrix,
                >(&json)?))
            }
            BeaconBlockType::Capella => {
                Ok(BeaconBlockBodyWrapper::Capella(serde_json::from_slice::<
                    BeaconBlockBodyCapella,
                >(&json)?))
            }
            BeaconBlockType::Deneb => Ok(BeaconBlockBodyWrapper::Deneb(serde_json::from_slice::<
                BeaconBlockBodyDeneb,
            >(&json)?)),
        }
    }

    /// Returns the inner [`BeaconBlockBody`].
    pub fn inner(self) -> BeaconBlockBody {
        match self {
            BeaconBlockBodyWrapper::Bellatrix(body) => BeaconBlockBody::Bellatrix(body),
            BeaconBlockBodyWrapper::Capella(body) => BeaconBlockBody::Capella(body),
            BeaconBlockBodyWrapper::Deneb(body) => BeaconBlockBody::Deneb(body),
        }
    }
}

#[superstruct(
    variants(Bellatrix, Capella, Deneb),
    variant_attributes(
        derive(
            serde::Deserialize,
            serde::Serialize,
            Clone,
            Debug,
            SimpleSerialize,
            Default
        ),
        serde(deny_unknown_fields)
    )
)]
#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
#[serde(untagged)]
pub struct BeaconBlockBody {
    randao_reveal: SignatureBytes,
    eth1_data: Eth1Data,
    graffiti: Bytes32,
    proposer_slashings: List<ProposerSlashing, 16>,
    attester_slashings: List<AttesterSlashing, 2>,
    attestations: List<Attestation, 128>,
    deposits: List<Deposit, 16>,
    voluntary_exits: List<SignedVoluntaryExit, 16>,
    sync_aggregate: SyncAggregate,
    #[serde(skip_serializing, default)]
    pub execution_payload: ExecutionPayload,
    #[superstruct(only(Capella, Deneb))]
    bls_to_execution_changes: List<SignedBlsToExecutionChange, 16>,
    #[superstruct(only(Deneb))]
    blob_kzg_commitments: List<ByteVector<48>, 4096>,
}

impl Default for BeaconBlockBody {
    fn default() -> Self {
        BeaconBlockBody::Bellatrix(BeaconBlockBodyBellatrix::default())
    }
}

superstruct_ssz!(BeaconBlockBody);

#[derive(Default, Clone, Debug, SimpleSerialize, serde::Deserialize, serde::Serialize)]
pub struct SignedBlsToExecutionChange {
    message: BlsToExecutionChange,
    signature: SignatureBytes,
}

#[derive(Default, Clone, Debug, SimpleSerialize, serde::Deserialize, serde::Serialize)]
pub struct BlsToExecutionChange {
    validator_index: U64,
    from_bls_pubkey: BLSPubKey,
    to_execution_address: Address,
}

/// Represents the different types of Execution payloads.
pub enum ExecutionPayloadWrapper {
    Bellatrix(ExecutionPayloadBellatrix),
    Capella(ExecutionPayloadCapella),
    Deneb(ExecutionPayloadDeneb),
}

impl ExecutionPayloadWrapper {
    /// Creates a new [`ExecutionPayloadWrapper`] from a JSON string, given the block type.
    pub fn new_from_json(json: Vec<u8>, block_type: &BeaconBlockType) -> Result<Self> {
        match block_type {
            BeaconBlockType::Bellatrix => Ok(ExecutionPayloadWrapper::Bellatrix(
                serde_json::from_slice::<ExecutionPayloadBellatrix>(&json)?,
            )),
            BeaconBlockType::Capella => {
                Ok(ExecutionPayloadWrapper::Capella(serde_json::from_slice::<
                    ExecutionPayloadCapella,
                >(&json)?))
            }
            BeaconBlockType::Deneb => Ok(ExecutionPayloadWrapper::Deneb(serde_json::from_slice::<
                ExecutionPayloadDeneb,
            >(&json)?)),
        }
    }

    /// Returns the inner [`ExecutionPayload`].
    pub fn inner(self) -> ExecutionPayload {
        match self {
            ExecutionPayloadWrapper::Bellatrix(payload) => ExecutionPayload::Bellatrix(payload),
            ExecutionPayloadWrapper::Capella(payload) => ExecutionPayload::Capella(payload),
            ExecutionPayloadWrapper::Deneb(payload) => ExecutionPayload::Deneb(payload),
        }
    }
}

#[superstruct(
    variants(Bellatrix, Capella, Deneb),
    variant_attributes(
        derive(
            serde::Deserialize,
            serde::Serialize,
            Debug,
            Default,
            SimpleSerialize,
            Clone
        ),
        serde(deny_unknown_fields)
    )
)]
#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
#[serde(untagged)]
pub struct ExecutionPayload {
    pub parent_hash: Bytes32,
    pub fee_recipient: Address,
    pub state_root: Bytes32,
    pub receipts_root: Bytes32,
    pub logs_bloom: LogsBloom,
    pub prev_randao: Bytes32,
    pub block_number: U64,
    pub gas_limit: U64,
    pub gas_used: U64,
    pub timestamp: U64,
    pub extra_data: ByteList<32>,
    #[serde(deserialize_with = "u256_deserialize")]
    pub base_fee_per_gas: U256,
    pub block_hash: Bytes32,
    pub transactions: List<Transaction, 1048576>,
    #[superstruct(only(Capella, Deneb))]
    withdrawals: List<Withdrawal, 16>,
    #[superstruct(only(Deneb))]
    blob_gas_used: U64,
    #[superstruct(only(Deneb))]
    excess_blob_gas: U64,
}

impl Default for ExecutionPayload {
    fn default() -> Self {
        ExecutionPayload::Bellatrix(ExecutionPayloadBellatrix::default())
    }
}

superstruct_ssz!(ExecutionPayload);

#[derive(Default, Clone, Debug, SimpleSerialize, serde::Deserialize, serde::Serialize)]
pub struct Withdrawal {
    index: U64,
    validator_index: U64,
    address: Address,
    amount: U64,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Default, SimpleSerialize, Clone)]
pub struct ProposerSlashing {
    signed_header_1: SignedBeaconBlockHeader,
    signed_header_2: SignedBeaconBlockHeader,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Default, SimpleSerialize, Clone)]
struct SignedBeaconBlockHeader {
    message: BeaconBlockHeader,
    signature: SignatureBytes,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Default, SimpleSerialize, Clone)]
struct BeaconBlockHeader {
    slot: U64,
    proposer_index: U64,
    parent_root: Bytes32,
    state_root: Bytes32,
    body_root: Bytes32,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Default, SimpleSerialize, Clone)]
pub struct AttesterSlashing {
    attestation_1: IndexedAttestation,
    attestation_2: IndexedAttestation,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Default, SimpleSerialize, Clone)]
struct IndexedAttestation {
    attesting_indices: List<U64, 2048>,
    data: AttestationData,
    signature: SignatureBytes,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Default, SimpleSerialize, Clone)]
pub struct Attestation {
    aggregation_bits: Bitlist<2048>,
    data: AttestationData,
    signature: SignatureBytes,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Default, SimpleSerialize, Clone)]
struct AttestationData {
    slot: U64,
    index: U64,
    beacon_block_root: Bytes32,
    source: Checkpoint,
    target: Checkpoint,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Default, SimpleSerialize, Clone)]
struct Checkpoint {
    epoch: U64,
    root: Bytes32,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Default, SimpleSerialize, Clone)]
pub struct SignedVoluntaryExit {
    message: VoluntaryExit,
    signature: SignatureBytes,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Default, SimpleSerialize, Clone)]
struct VoluntaryExit {
    epoch: U64,
    validator_index: U64,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Default, SimpleSerialize, Clone)]
pub struct Deposit {
    proof: Vector<Bytes32, 33>,
    data: DepositData,
}

#[derive(serde::Deserialize, serde::Serialize, Default, Debug, SimpleSerialize, Clone)]
struct DepositData {
    pubkey: BLSPubKey,
    withdrawal_credentials: Bytes32,
    amount: U64,
    signature: SignatureBytes,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Default, SimpleSerialize, Clone)]
pub struct Eth1Data {
    deposit_root: Bytes32,
    deposit_count: U64,
    block_hash: Bytes32,
}

#[derive(serde::Deserialize, Debug)]
pub struct Bootstrap {
    #[serde(deserialize_with = "header_deserialize")]
    pub header: Header,
    pub current_sync_committee: SyncCommittee,
    pub current_sync_committee_branch: Vec<Bytes32>,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, Default)]
pub struct Update {
    #[serde(deserialize_with = "header_deserialize")]
    pub attested_header: Header,
    pub next_sync_committee: SyncCommittee,
    pub next_sync_committee_branch: Vec<Bytes32>,
    #[serde(deserialize_with = "header_deserialize")]
    pub finalized_header: Header,
    pub finality_branch: Vec<Bytes32>,
    pub sync_aggregate: SyncAggregate,
    pub signature_slot: U64,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Default)]
pub struct FinalityUpdate {
    #[serde(deserialize_with = "header_deserialize")]
    pub attested_header: Header,
    #[serde(deserialize_with = "header_deserialize")]
    pub finalized_header: Header,
    pub finality_branch: Vec<Bytes32>,
    pub sync_aggregate: SyncAggregate,
    pub signature_slot: U64,
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct OptimisticUpdate {
    #[serde(deserialize_with = "header_deserialize")]
    pub attested_header: Header,
    pub sync_aggregate: SyncAggregate,
    pub signature_slot: U64,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Default, SimpleSerialize)]
pub struct Header {
    pub slot: U64,
    pub proposer_index: U64,
    pub parent_root: Bytes32,
    pub state_root: Bytes32,
    pub body_root: Bytes32,
}

#[derive(Debug, Clone, Default, SimpleSerialize, serde::Deserialize, serde::Serialize)]
pub struct SyncCommittee {
    pub pubkeys: Vector<BLSPubKey, 512>,
    pub aggregate_pubkey: BLSPubKey,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, Default, SimpleSerialize)]
pub struct SyncAggregate {
    pub sync_committee_bits: Bitvector<512>,
    pub sync_committee_signature: SignatureBytes,
}

pub struct GenericUpdate {
    pub attested_header: Header,
    pub sync_aggregate: SyncAggregate,
    pub signature_slot: u64,
    pub next_sync_committee: Option<SyncCommittee>,
    pub next_sync_committee_branch: Option<Vec<Bytes32>>,
    pub finalized_header: Option<Header>,
    pub finality_branch: Option<Vec<Bytes32>>,
}

impl From<&Update> for GenericUpdate {
    fn from(update: &Update) -> Self {
        Self {
            attested_header: update.attested_header.clone(),
            sync_aggregate: update.sync_aggregate.clone(),
            signature_slot: update.signature_slot.into(),
            next_sync_committee: Some(update.next_sync_committee.clone()),
            next_sync_committee_branch: Some(update.next_sync_committee_branch.clone()),
            finalized_header: Some(update.finalized_header.clone()),
            finality_branch: Some(update.finality_branch.clone()),
        }
    }
}

impl From<&FinalityUpdate> for GenericUpdate {
    fn from(update: &FinalityUpdate) -> Self {
        Self {
            attested_header: update.attested_header.clone(),
            sync_aggregate: update.sync_aggregate.clone(),
            signature_slot: update.signature_slot.into(),
            next_sync_committee: None,
            next_sync_committee_branch: None,
            finalized_header: Some(update.finalized_header.clone()),
            finality_branch: Some(update.finality_branch.clone()),
        }
    }
}

impl From<&OptimisticUpdate> for GenericUpdate {
    fn from(update: &OptimisticUpdate) -> Self {
        Self {
            attested_header: update.attested_header.clone(),
            sync_aggregate: update.sync_aggregate.clone(),
            signature_slot: update.signature_slot.into(),
            next_sync_committee: None,
            next_sync_committee_branch: None,
            finalized_header: None,
            finality_branch: None,
        }
    }
}

/// Holds an aggregate of all update types that are necessary to verify and apply a new Ethereum
/// state.
#[derive(Debug)]
pub struct AggregateUpdates {
    pub updates: Vec<Update>,
    pub finality_update: FinalityUpdate,
    pub optimistic_update: OptimisticUpdate,
}

impl BeaconBlockBody {
    /// Creates a new [`BeaconBlockBody`] from a JSON string and an existing Execution Payload.
    pub fn new_from_existing_with_execution_payload(
        body: BeaconBlockBody,
        payload: ExecutionPayload,
    ) -> Self {
        match body {
            BeaconBlockBody::Bellatrix(beacon_block_body) => {
                let mut beacon_block_body_bellatrix: BeaconBlockBodyBellatrix = beacon_block_body;
                beacon_block_body_bellatrix.execution_payload = payload;
                BeaconBlockBody::Bellatrix(beacon_block_body_bellatrix)
            }
            BeaconBlockBody::Capella(beacon_block_body) => {
                let mut beacon_block_body_capella: BeaconBlockBodyCapella = beacon_block_body;
                beacon_block_body_capella.execution_payload = payload;
                BeaconBlockBody::Capella(beacon_block_body_capella)
            }
            BeaconBlockBody::Deneb(beacon_block_body) => {
                let mut beacon_block_body_deneb: BeaconBlockBodyDeneb = beacon_block_body;
                beacon_block_body_deneb.execution_payload = payload;
                BeaconBlockBody::Deneb(beacon_block_body_deneb)
            }
        }
    }

    /// Returns the block type of the [`BeaconBlockBody`].
    pub fn get_block_type(&self) -> BeaconBlockType {
        match self {
            BeaconBlockBody::Bellatrix(_) => BeaconBlockType::Bellatrix,
            BeaconBlockBody::Capella(_) => BeaconBlockType::Capella,
            BeaconBlockBody::Deneb(_) => BeaconBlockType::Deneb,
        }
    }
}
