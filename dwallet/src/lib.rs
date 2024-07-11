//! Ethereum Light Client implementation for dWallet Network
//!
//! The `EthState` struct is created to have only the necessary functions for the proof
//! verification. Some functions in this struct are borrowed from the consensus module in the Helios
//! project. We had to copy them since their logic is similar, but the types they operate on are
//! different. Original code can be found here: `consensus/src/consensus.rs` inside `Inner` struct
//! impl block.

pub mod eth_state;
pub mod light_client;
pub mod utils;
