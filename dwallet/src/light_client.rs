//! This module defines an Ethereum light client interface for the dwallet network, providing
//! functionality to initialize, start, and interact with an Ethereum light client. It supports
//! retrieving Merkle proofs for account and storage states and fetching updates from the consensus
//! layer.
//!
//! The main structure provided is [`EthLightClientWrapper`], which integrates with the `ethers`
//! library for Ethereum interaction. The configuration and request parameters for the client are
//! defined in the [`EthLightClientConfig`] and [`ProofRequestParameters`] structures respectively.

use anyhow::anyhow;
use client::{Client, ClientBuilder};
use config::Network;
use consensus::database::FileDB;
use ethers::prelude::{Address, EIP1186ProofResponse};
use execution::types::ProofVerificationInput;

/// Interface of the Ethereum light client for dWallet network.
pub struct EthLightClientWrapper {
    client: Client<FileDB>,
}

#[derive(Default, Clone)]
pub struct EthLightClientConfig {
    // Eth Network (Mainnet, Goerli, etc).
    pub network: Network,
    // Eth RPC URL.
    pub execution_rpc: String,
    // Consensus RPC URL.
    pub consensus_rpc: String,
    // Checkpoint.
    pub checkpoint: String,
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

impl EthLightClientWrapper {
    fn new(config: EthLightClientConfig) -> Result<Self, anyhow::Error> {
        let network = &config.network;
        let client: Client<FileDB> = ClientBuilder::new()
            .network(*network)
            .execution_rpc(&config.execution_rpc)
            .consensus_rpc(&config.consensus_rpc)
            .checkpoint(&config.checkpoint)
            .data_dir("/tmp/helios".parse()?)
            .build()
            .map_err(|e| anyhow!("failed to create a client: {}", e))?;

        Ok(Self { client })
    }

    async fn start(&mut self) -> Result<(), anyhow::Error> {
        self.client
            .start()
            .await
            .map_err(|e| anyhow!("failed to start a client: {}", e))?;
        self.client.wait_synced().await;
        Ok(())
    }

    /// Initializes a new Ethereum light client.
    ///
    /// Creates a new instance of [`EthLightClientWrapper`] using the provided
    /// configuration.
    /// It constructs the client by calling the `new` method,
    /// which also syncs the state of the client.
    /// If successful, it returns the initialized [`EthLightClientWrapper`] instance.
    ///
    /// # Arguments
    /// * `eth_client_config` – A configuration struct for the Ethereum light client.
    pub async fn init_new_light_client(
        eth_client_config: EthLightClientConfig,
    ) -> Result<EthLightClientWrapper, anyhow::Error> {
        let mut eth_lc = EthLightClientWrapper::new(eth_client_config.clone())?;
        eth_lc.start().await?;
        Ok(eth_lc)
    }

    /// Get the Merkle Tree Proof (EIP1186Proof) for the client parameters.
    pub async fn get_proofs(
        self: &mut EthLightClientWrapper,
        contract_addr: &Address,
        proof_parameters: ProofRequestParameters,
        latest_execution_block_number: u64,
    ) -> Result<EIP1186ProofResponse, anyhow::Error> {
        let message_map_index = execution::get_message_storage_slot(
            proof_parameters.message.clone(),
            proof_parameters.dwallet_id.clone(),
            proof_parameters.data_slot,
        )
        .map_err(|e| anyhow!("failed to calculate message storage slot: {}", e))?;

        let proof = self
            .client
            .get_proof(
                contract_addr,
                &[message_map_index],
                latest_execution_block_number,
            )
            .await
            .map_err(|e| anyhow!("failed to get proof: {}", e))?;

        Ok(proof)
    }
}
