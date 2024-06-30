//! This module defines an Ethereum light client interface for the dwallet network, providing functionality
//! to initialize, start, and interact with an Ethereum light client. It supports retrieving Merkle proofs
//! for account and storage states and fetching updates from the consensus layer.
//!
//! The main structure provided is `EthLightClient`, which integrates with the `ethers` library for Ethereum
//! interaction. The configuration and request parameters for the client are defined in
//! the `EthLightClientConfig` and `ProofRequestParameters` structures respectively.

use anyhow::{anyhow};
use client::{Client, ClientBuilder};
use config::Network;
use consensus::database::FileDB;
use consensus::types::{UpdatesResponse};
use ethers::prelude::{Address};
use consensus::rpc::ConsensusRpc;
use consensus::rpc::nimbus_rpc::NimbusRpc;
use execution::types::ProofVerificationInput;
use crate::eth_state::EthState;
use crate::utils::{create_account_proof, extract_storage_proof};

/// Interface of Ethereum light client for dwallet network
pub struct EthLightClient {
    client: Client<FileDB>,
    eth_state: EthState,
    consensus_rpc: NimbusRpc,
}

#[derive(Default, Clone)]
pub struct EthLightClientConfig {
    // Eth Network (Mainnet, Goerli, etc).
    pub network: Network,
    // Eth RPC URL.
    pub execution_rpc: String,
    // Consensus RPC URL.
    pub consensus_rpc: String,
    pub max_checkpoint_age: u64,
}

#[derive(Default, Clone)]
pub struct ProofRequestParameters {
    pub message: String,
    pub dwallet_id: Vec<u8>,
    pub data_slot: u64,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct ProofResponse {
    pub account_proof: ProofVerificationInput,
    pub storage_proof: ProofVerificationInput,
}

impl EthLightClient {
    fn new(config: EthLightClientConfig, eth_state: EthState) -> Result<Self, anyhow::Error> {
        let network = &config.network;
        let client: Client<FileDB> = ClientBuilder::new()
            .network(network.clone())
            .execution_rpc(&config.execution_rpc)
            .consensus_rpc(&config.consensus_rpc)
            .checkpoint(&eth_state.last_checkpoint)
            .data_dir("/tmp/helios".parse()?)
            .build()
            .map_err(|e| anyhow!("failed to create client: {}", e))?;

        let consensus_rpc = NimbusRpc::new(&config.consensus_rpc);

        Ok(Self {
            client,
            eth_state,
            consensus_rpc,
        })
    }

    async fn start(&mut self) -> Result<(), anyhow::Error> {
        self.client
            .start()
            .await
            .map_err(|e| anyhow!("failed to start client: {}", e))?;
        self.client.wait_synced().await;
        Ok(())
    }

    pub async fn init_new_light_client(
        eth_client_config: EthLightClientConfig,
        eth_state: EthState,
    ) -> Result<EthLightClient, anyhow::Error> {
        let mut eth_lc = EthLightClient::new(eth_client_config.clone(), eth_state)?;
        eth_lc.start().await?;
        Ok(eth_lc)
    }

    /// Get the Merkle Tree Proof (EIP1186Proof) for the client parameters.
    pub async fn get_proofs(
        self: &mut EthLightClient,
        contract_addr: &Address,
        proof_parameters: ProofRequestParameters,
    ) -> Result<ProofResponse, anyhow::Error> {
        let block_number = self.eth_state.last_update_execution_block_number;
        let state_root = &self.eth_state.last_update_execution_state_root;
        let message_map_index = execution::utils::get_message_storage_slot(
            proof_parameters.message.clone(),
            proof_parameters.dwallet_id.clone(),
            proof_parameters.data_slot,
        )
            .map_err(|e| anyhow!("failed to calculate message storage slot: {}", e))?;

        let proof = self
            .client
            .get_proof(contract_addr, &[message_map_index], block_number)
            .await
            .map_err(|e| anyhow!("failed to get proof: {}", e))?;

        let account_proof = create_account_proof(contract_addr, state_root, &proof);

        let storage_proof = extract_storage_proof(message_map_index, proof)
            .map_err(|e| anyhow!("failed to create storage proof: {}", e))?;

        Ok(ProofResponse {
            account_proof,
            storage_proof,
        })
    }

    /// Get new block headers' updates from the consensus layer.
    /// The updates are fetched starting from the last validated checkpoint
    /// (last_checkpoint field) that is stored in the state.
    pub async fn get_updates(
        &mut self,
    ) -> Result<UpdatesResponse, eyre::Error> {
        self.eth_state.get_updates(&self.consensus_rpc).await
    }
}

