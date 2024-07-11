use std::cmp;
use chrono::{Duration};
use std::time::{SystemTime, UNIX_EPOCH};
use ethers::utils::hex::ToHexExt;
use eyre::{anyhow, Error, eyre};
use milagro_bls::PublicKey;
use ssz_rs::{Merkleized, Node, Vector};
use tracing::info;

/// The EthState struct is designed to maintain the state Ethereum's consensus layer, and perform
/// various operations on it.
/// Operations include synchronizing the local state with the blockchain state, verifying and
/// applying updates, etc.
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Default)]
pub struct EthState {
    #[serde(default)]
    pub last_checkpoint: String,
    #[serde(default)]
    pub latest_header: Header,
    #[serde(default)]
    pub current_sync_committee: SyncCommittee,
    #[serde(default)]
    pub next_sync_committee: Option<SyncCommittee>,
    #[serde(default)]
    pub finalized_header: Header,
    #[serde(default)]
    optimistic_header: Header,
    #[serde(default)]
    previous_max_active_participants: u64,
    #[serde(default)]
    current_max_active_participants: u64,
    #[serde(default)]
    network: Network,
    #[serde(default)]
    pub last_update_execution_block_number: u64,
    #[serde(default)]
    pub last_update_execution_state_root: Bytes32,
}

impl EthState {
    pub fn new() -> Self {
        EthState {
            last_checkpoint: String::default(),
            current_sync_committee: SyncCommittee::default(),
            next_sync_committee: None,
            finalized_header: Header::default(),
            optimistic_header: Header::default(),
            previous_max_active_participants: u64::default(),
            current_max_active_participants: u64::default(),
            network: Network::default(),
            latest_header: Header::default(),
            last_update_execution_block_number: u64::default(),
            last_update_execution_state_root: Bytes32::default(),
        }
    }

    /// Sets the checkpoint for the Ethereum state.
    ///
    /// Set the last known checkpoint for the Ethereum state. The checkpoint
    /// is a string that represents a specific point in the blockchain history, typically a
    /// beacon block hash.
    pub fn set_checkpoint(&mut self, checkpoint: String) -> Self {
        self.last_checkpoint = checkpoint;
        self.clone()
    }

    /// Sets the network for the Ethereum state.
    ///
    /// Set the network for the Ethereum state. The network is an enum that
    /// represents the specific Ethereum network (e.g., Mainnet, Holesky, devnet, etc.).
    pub fn set_network(&mut self, network: Network) -> Self {
        self.network = network;
        self.clone()
    }

    /// Synchronizes the local state with the blockchain state based on a given checkpoint.
    /// Performs a multi-step process to ensure the local state is up-to-date with
    /// the blockchain's state.
    ///
    /// # Arguments
    /// * `checkpoint`: A `&str` slice that represents the checkpoint from which to start the
    ///   synchronization process. Typically, this would be a block hash or a similar identifier
    ///   that marks a specific point in the blockchain history.
    ///
    /// # Process
    ///
    /// 1. **Bootstrap:** Initializes the synchronization process using the provided checkpoint.
    ///    This step involves setting up the local state to match the state at the checkpoint.
    ///
    /// 2. **Fetch Updates:** Retrieves updates from the blockchain for the current period. The
    ///    current period is calculated based on the slot of the last finalized header.
    ///
    /// 3. **Verify and Apply Updates:**
    ///    - For each update fetched, it first verifies the update for correctness and then applies
    ///      the update to the local state.
    ///    - Verifies and applies a finality update, which includes updates that have been finalized
    ///      and are irreversible.
    ///    - Verifies and applies an optimistic update, which might still be subject to change but
    ///      is accepted optimistically to keep the state as current as possible.
    pub async fn get_updates(
        &mut self,
        rpc: &NimbusRpc,
    ) -> Result<UpdatesResponse, eyre::Error> {
        let checkpoint = self.last_checkpoint.clone();
        if self.finalized_header.slot == U64::from(0)
            || self.current_sync_committee.aggregate_pubkey == BLSPubKey::default()
        {
            self.bootstrap(&rpc, &checkpoint).await?;
        }

        let current_period = calc_sync_period(self.finalized_header.slot.into());
        let updates = rpc
            .get_updates(current_period, MAX_REQUEST_LIGHT_CLIENT_UPDATES)
            .await?;

        let finality_update = rpc.get_finality_update().await?;

        let optimistic_update = rpc.get_optimistic_update().await?;

        let (execution_block_number, execution_state_root) = self
            .get_execution_block_info_from_update(&finality_update, rpc)
            .await?;

        self.last_update_execution_block_number = execution_block_number;
        self.last_update_execution_state_root = execution_state_root;

        Ok(AggregateUpdates {
            updates,
            finality_update,
            optimistic_update,
        })
    }

    async fn get_execution_block_info_from_update(
        &self,
        update: &FinalityUpdate,
        rpc: &NimbusRpc,
    ) -> Result<(u64, Bytes32), Error> {
        let latest_header_slot = update.attested_header.slot.as_u64();
        let block = rpc.get_block(latest_header_slot).await?;

        Ok((
            (*block.body.execution_payload().block_number()).into(),
            block.body.execution_payload().state_root().clone(),
        ))
    }

    /// Verifies and applies updates to the Ethereum state.
    /// This function takes a reference to an `AggregateUpdates` which contains updates fetched from
    /// the blockchain. It iterates over each update, verifies it for correctness and then
    /// applies it to the local state. The function performs these operations for three types of
    /// updates: regular updates, finality updates, and optimistic updates. # Arguments
    /// * `updates`: A reference to an `AggregateUpdates` object that contains the updates to be
    ///   verified and applied.
    /// # Returns
    /// * `Result<(), Error>`: This function returns a `Result` type. On successful verification and
    ///   application of all updates, it returns `Ok(())`. If there is an error at any point during
    ///   the verification or application process, it returns `Err(Error)`.
    /// # Errors
    /// This function will return an error if:
    /// * Any of the updates fails the verification process.
    /// * There is an error while applying any of the updates.
    pub fn verify_and_apply_updates(&mut self, updates: &UpdatesResponse) -> Result<(), Error> {
        for update in &updates.updates {
            self.verify_update(update)?;
            self.apply_update(update);
        }

        self.verify_finality_update(&updates.finality_update)?;
        self.apply_finality_update(&updates.finality_update);

        self.verify_optimistic_update(&updates.optimistic_update)?;
        self.apply_optimistic_update(&updates.optimistic_update);

        Ok(())
    }

    // todo(yuval): explain why code duplications for next functions
    /// Initializes the synchronization process using the provided checkpoint.
    /// This function takes a reference to a `NimbusRpc` and a checkpoint string. It fetches the
    /// bootstrap data from the blockchain using the provided checkpoint and verifies it for
    /// correctness. If the bootstrap data is valid, it updates the local state to match the
    /// state at the checkpoint. # Arguments
    /// * `rpc`: A reference to a `NimbusRpc` object that is used to interact with the consensus
    ///   layer of the blockchain.
    /// * `checkpoint`: A `&str` slice that represents the checkpoint from which to start the
    ///   synchronization process. Typically, this would be a beacon block hash.
    /// # Returns
    /// * `Result<(), Error>`: This function returns a `Result` type. If the bootstrap data is
    ///   successfully fetched and verified, and the local state is successfully updated, it returns
    ///   `Ok(())`. If there is an error at any point during the process, it returns `Err(Error)`.
    /// # Errors
    /// This function will return an error if:
    /// * The bootstrap data could not be fetched from the blockchain.
    /// * The checkpoint is too old.
    /// * The header hash of the bootstrap data does not match the checkpoint.
    /// * The current sync committee proof is invalid.
    pub async fn bootstrap(&mut self, rpc: &NimbusRpc, checkpoint: &str) -> Result<(), Error> {
        let mut bootstrap: Bootstrap = rpc
            .get_bootstrap(hex::decode(&checkpoint[2..])?.as_slice())
            .await
            .map_err(|_| eyre!("could not fetch bootstrap"))?;

        let is_valid = self.is_valid_checkpoint(bootstrap.header.slot.into());
        if !is_valid {
            return Err(eyre!(
                "checkpoint too old, consider using a more recent block"
            ));
        }

        let committee_valid = is_current_committee_proof_valid(
            &bootstrap.header,
            &mut bootstrap.current_sync_committee,
            &bootstrap.current_sync_committee_branch,
        );

        let header_hash = bootstrap.header.hash_tree_root()?.to_string();
        let expected_hash = checkpoint.to_string();
        let header_valid = header_hash == checkpoint;

        if !header_valid {
            return Err(ConsensusError::InvalidHeaderHash(expected_hash, header_hash).into());
        }

        if !committee_valid {
            return Err(ConsensusError::InvalidCurrentSyncCommitteeProof.into());
        }

        self.finalized_header = bootstrap.header.clone();
        self.current_sync_committee = bootstrap.current_sync_committee;

        Ok(())
    }

    fn apply_update(&mut self, update: &Update) {
        let update = GenericUpdate::from(update);
        self.apply_generic_update(&update);
    }

    fn apply_finality_update(&mut self, update: &FinalityUpdate) {
        let update = GenericUpdate::from(update);
        self.apply_generic_update(&update);
    }

    fn log_finality_update(&self, update: &GenericUpdate) {
        let participation =
            get_bits(&update.sync_aggregate.sync_committee_bits) as f32 / 512f32 * 100f32;
        let decimals = if participation == 100.0 { 1 } else { 2 };
        let age = self.age(self.finalized_header.slot.as_u64());

        info!(
            target: "helios::consensus",
            "finalized slot             slot={}  confidence={:.decimals$}%  age={:02}:{:02}:{:02}:{:02}",
            self.finalized_header.slot.as_u64(),
            participation,
            age.num_days(),
            age.num_hours() % 24,
            age.num_minutes() % 60,
            age.num_seconds() % 60,
        );
    }

    fn apply_optimistic_update(&mut self, update: &OptimisticUpdate) {
        let update = GenericUpdate::from(update);
        self.apply_generic_update(&update);
    }

    fn log_optimistic_update(&self, update: &GenericUpdate) {
        let participation =
            get_bits(&update.sync_aggregate.sync_committee_bits) as f32 / 512f32 * 100f32;
        let decimals = if participation == 100.0 { 1 } else { 2 };
        let age = self.age(self.optimistic_header.slot.as_u64());

        info!(
            target: "helios::consensus",
            "updated head               slot={}  confidence={:.decimals$}%  age={:02}:{:02}:{:02}:{:02}",
            self.optimistic_header.slot.as_u64(),
            participation,
            age.num_days(),
            age.num_hours() % 24,
            age.num_minutes() % 60,
            age.num_seconds() % 60,
        );
    }

    fn age(&self, slot: u64) -> Duration {
        let expected_time = self.slot_timestamp(slot);
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        let delay = now - std::time::Duration::from_secs(expected_time);
        Duration::from_std(delay).unwrap()
    }

    /// Applies a generic update to the consensus client's state.
    /// Processes an update received from the network, updating the client's
    /// internal state based on the contents of the update. It handles the update by performing
    /// several checks and applying changes to the consensus client's tracked headers and
    /// sync committees based on these updates.
    /// # Behavior
    /// The function operates as follows:
    /// 1. **Active Participants Update:** Updates the count of current maximum active participants
    ///    based on the sync committee bits provided in the update.
    /// 2. **Optimistic Header Update:** If the update passes the safety threshold and the attested
    ///    header slot is greater than the currently optimistic header slot, the optimistic header
    ///    is updated to the new attested header.
    /// 3. **Sync Committee and Finality Checks:** Determines whether the update should be applied
    ///    based on several criteria, including whether it has a majority of committee bits, whether
    ///    it references a newer finalized slot than the current state, and whether it aligns with
    ///    the current sync committee period.
    /// 4. **State Update:** If the update should be applied, the function updates the current and
    ///    next sync committees, the finalized header, and potentially the optimistic header.
    /// # Important Considerations
    /// - This function assumes that the update has already been verified for correctness and
    ///   authenticity.
    /// - It makes decisions based on comparing the update's slots and periods against the client's
    ///   current state, ensuring that only relevant and newer updates are applied.
    fn apply_generic_update(&mut self, update: &GenericUpdate) {
        let committee_bits = get_bits(&update.sync_aggregate.sync_committee_bits);

        self.latest_header = update.attested_header.clone();

        self.current_max_active_participants =
            u64::max(self.current_max_active_participants, committee_bits);

        let should_update_optimistic = committee_bits > self.safety_threshold()
            && update.attested_header.slot > self.optimistic_header.slot;

        if should_update_optimistic {
            self.optimistic_header = update.attested_header.clone();
            self.log_optimistic_update(update);
        }

        let update_attested_period = calc_sync_period(update.attested_header.slot.into());

        let update_finalized_slot = update
            .finalized_header
            .as_ref()
            .map(|h| h.slot.as_u64())
            .unwrap_or(0);

        let update_finalized_period = calc_sync_period(update_finalized_slot);

        let update_has_finalized_next_committee = self.next_sync_committee.is_none()
            && self.has_sync_update(update)
            && self.has_finality_update(update)
            && update_finalized_period == update_attested_period;

        let should_apply_update = {
            let has_majority = committee_bits * 3 >= 512 * 2;
            if !has_majority {
                tracing::warn!("skipping block with low vote count");
            }

            let update_is_newer = update_finalized_slot > self.finalized_header.slot.as_u64();
            let good_update = update_is_newer || update_has_finalized_next_committee;

            has_majority && good_update
        };

        if should_apply_update {
            let store_period = calc_sync_period(self.finalized_header.slot.into());

            if self.next_sync_committee.is_none() {
                self.next_sync_committee = update.next_sync_committee.clone();
            } else if update_finalized_period == store_period + 1 {
                info!(target: "helios::consensus", "sync committee updated");
                self.current_sync_committee = self.next_sync_committee.clone().unwrap();
                self.next_sync_committee = update.next_sync_committee.clone();
                self.previous_max_active_participants = self.current_max_active_participants;
                self.current_max_active_participants = 0;
            }

            if update_finalized_slot > self.finalized_header.slot.as_u64() {
                self.finalized_header = update.finalized_header.clone().unwrap();
                self.log_finality_update(update);

                if self.finalized_header.slot.as_u64() % 32 == 0 {
                    let checkpoint_res = self.finalized_header.hash_tree_root();
                    if let std::prelude::rust_2015::Ok(checkpoint) = checkpoint_res {
                        self.last_checkpoint = format!("0x{:?}", checkpoint.as_ref());
                    }
                }

                if self.finalized_header.slot > self.optimistic_header.slot {
                    self.optimistic_header = self.finalized_header.clone();
                }
            }

            let finalized_header_checkpoint = self
                .finalized_header
                .hash_tree_root()
                .map_err(|_| anyhow!("could not hash finalized header"))
                .unwrap()
                .encode_hex_with_prefix();

            self.last_checkpoint = finalized_header_checkpoint;
        }
    }

    fn verify_update(&self, update: &Update) -> Result<(), Error> {
        let update = GenericUpdate::from(update);
        self.verify_generic_update(&update)
    }

    fn verify_finality_update(&self, update: &FinalityUpdate) -> Result<(), Error> {
        let update = GenericUpdate::from(update);
        self.verify_generic_update(&update)
    }

    fn verify_optimistic_update(&self, update: &OptimisticUpdate) -> Result<(), Error> {
        let update = GenericUpdate::from(update);
        self.verify_generic_update(&update)
    }

    /// Verifies the correctness of a generic update received by the consensus client.
    /// Validates a `GenericUpdate` based on several criteria to ensure it can be safely applied to
    /// the client's state. The verification process includes checks for sufficient
    /// participation, timing and period validity, relevance of the update, and the authenticity
    /// of signatures.
    ///
    /// # Verification Process
    /// 1. **Participation Check:** Verifies that the update has sufficient participation from the
    ///    sync committee by checking the number of bits set in
    ///    `sync_aggregate.sync_committee_bits`.
    /// 2. **Timing Validation:** Ensures the update's timing is valid by comparing the
    ///    `signature_slot` with the `attested_header.slot` and the `finalized_header.slot`. The
    ///    update must be signed after the attested header's slot and before or at the current slot,
    ///    and it must reference a slot that is not older than the last finalized slot.
    /// 3. **Period Validation:** Confirms that the update's signature slot falls within the correct
    ///    sync committee period, allowing for updates from the current or immediately next period
    ///    if the next sync committee is known.
    /// 4. **Relevance Check:** Ensures the update is relevant by disallowing updates that reference
    ///    slots older than the last finalized slot unless introducing a new sync committee.
    /// 5. **Finality and Next Sync Committee Proofs:** Validates proofs related to the update's
    ///    finality and the inclusion of a new sync committee, if present.
    /// 6. **Signature Verification:** Confirms that the sync committee's signature on the attested
    ///    header is valid, indicating agreement with the header's contents.
    fn verify_generic_update(&self, update: &GenericUpdate) -> Result<(), Error> {
        let bits = get_bits(&update.sync_aggregate.sync_committee_bits);
        if bits == 0 {
            return Err(ConsensusError::InsufficientParticipation.into());
        }

        let update_finalized_slot = update.finalized_header.clone().unwrap_or_default().slot;
        let valid_time = self.expected_current_slot() >= update.signature_slot
            && update.signature_slot > update.attested_header.slot.as_u64()
            && update.attested_header.slot >= update_finalized_slot;

        if !valid_time {
            return Err(ConsensusError::InvalidTimestamp.into());
        }

        let store_period = calc_sync_period(self.finalized_header.slot.into());
        let update_sig_period = calc_sync_period(update.signature_slot);
        let valid_period = if self.next_sync_committee.is_some() {
            update_sig_period == store_period || update_sig_period == store_period + 1
        } else {
            update_sig_period == store_period
        };

        if !valid_period {
            return Err(ConsensusError::InvalidPeriod.into());
        }

        let update_attested_period = calc_sync_period(update.attested_header.slot.into());
        let update_has_next_committee = self.next_sync_committee.is_none()
            && update.next_sync_committee.is_some()
            && update_attested_period == store_period;

        if update.attested_header.slot <= self.finalized_header.slot && !update_has_next_committee {
            return Err(ConsensusError::NotRelevant.into());
        }

        if update.finalized_header.is_some() && update.finality_branch.is_some() {
            let is_valid = is_finality_proof_valid(
                &update.attested_header,
                &mut update.finalized_header.clone().unwrap(),
                &update.finality_branch.clone().unwrap(),
            );

            if !is_valid {
                return Err(ConsensusError::InvalidFinalityProof.into());
            }
        }

        if update.next_sync_committee.is_some() && update.next_sync_committee_branch.is_some() {
            let is_valid = is_next_committee_proof_valid(
                &update.attested_header,
                &mut update.next_sync_committee.clone().unwrap(),
                &update.next_sync_committee_branch.clone().unwrap(),
            );

            if !is_valid {
                return Err(ConsensusError::InvalidNextSyncCommitteeProof.into());
            }
        }

        let sync_committee = if update_sig_period == store_period {
            &self.current_sync_committee
        } else {
            self.next_sync_committee.as_ref().unwrap()
        };

        let pks =
            get_participating_keys(sync_committee, &update.sync_aggregate.sync_committee_bits)?;

        let is_valid_sig = self.verify_sync_committee_signture(
            &pks,
            &update.attested_header,
            &update.sync_aggregate.sync_committee_signature,
            update.signature_slot,
        );

        if !is_valid_sig {
            return Err(ConsensusError::InvalidSignature.into());
        }

        Ok(())
    }

    fn expected_current_slot(&self) -> u64 {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        let genesis_time = self.network.to_base_config().chain.genesis_time;
        let since_genesis = now - std::time::Duration::from_secs(genesis_time);

        since_genesis.as_secs() / 12
    }

    fn has_finality_update(&self, update: &GenericUpdate) -> bool {
        update.finalized_header.is_some() && update.finality_branch.is_some()
    }

    fn has_sync_update(&self, update: &GenericUpdate) -> bool {
        update.next_sync_committee.is_some() && update.next_sync_committee_branch.is_some()
    }

    fn safety_threshold(&self) -> u64 {
        cmp::max(
            self.current_max_active_participants,
            self.previous_max_active_participants,
        ) / 2
    }

    fn verify_sync_committee_signture(
        &self,
        pks: &[PublicKey],
        attested_header: &Header,
        signature: &SignatureBytes,
        signature_slot: u64,
    ) -> bool {
        let res: Result<bool, Error> = (move || {
            let pks: Vec<&PublicKey> = pks.iter().collect();
            let header_root =
                Bytes32::try_from(attested_header.clone().hash_tree_root()?.as_ref())?;
            let signing_root = self.compute_committee_sign_root(header_root, signature_slot)?;

            Ok(is_aggregate_valid(signature, signing_root.as_ref(), &pks))
        })();

        res.unwrap_or(false)
    }

    fn compute_committee_sign_root(&self, header: Bytes32, slot: u64) -> Result<Node, Error> {
        let genesis_root = self
            .network
            .to_base_config()
            .chain
            .genesis_root
            .to_vec()
            .try_into()
            .unwrap();

        let domain_type = &hex::decode("07000000")?[..];
        let fork_version =
            Vector::try_from(Self::fork_version(self.network, slot)).map_err(|(_, err)| err)?;
        let domain = compute_domain(domain_type, fork_version, genesis_root)?;
        compute_signing_root(header, domain)
    }

    fn fork_version(network: Network, slot: u64) -> Vec<u8> {
        let epoch = slot / 32;
        let config = network.to_base_config();

        if epoch >= config.forks.deneb.epoch {
            config.forks.deneb.fork_version.clone()
        } else if epoch >= config.forks.capella.epoch {
            config.forks.capella.fork_version.clone()
        } else if epoch >= config.forks.bellatrix.epoch {
            config.forks.bellatrix.fork_version.clone()
        } else if epoch >= config.forks.altair.epoch {
            config.forks.altair.fork_version.clone()
        } else {
            config.forks.genesis.fork_version.clone()
        }
    }

    // Determines blockhash_slot age and returns true if it is less than 14 days old
    fn is_valid_checkpoint(&self, blockhash_slot: u64) -> bool {
        let current_slot = self.expected_current_slot();
        let current_slot_timestamp = self.slot_timestamp(current_slot);
        let blockhash_slot_timestamp = self.slot_timestamp(blockhash_slot);

        let slot_age = current_slot_timestamp
            .checked_sub(blockhash_slot_timestamp)
            .unwrap_or_default();
        slot_age < self.network.to_base_config().max_checkpoint_age
    }

    fn slot_timestamp(&self, slot: u64) -> u64 {
        slot * 12 + self.network.to_base_config().chain.genesis_time
    }
}
