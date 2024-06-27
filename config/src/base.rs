use serde::{Deserialize, Serialize};
use std::default::Default;
use std::fs;
use std::net::{IpAddr, Ipv4Addr};
use std::path::PathBuf;
use eyre::Report;

use crate::utils::bytes_deserialize;
use crate::types::{ChainConfig, Forks};
use crate::utils::bytes_serialize;

/// The base configuration for a network.
#[derive(Serialize, Deserialize)]
pub struct BaseConfig {
    pub rpc_bind_ip: IpAddr,
    pub rpc_port: u16,
    pub consensus_rpc: Option<String>,
    #[serde(
        deserialize_with = "bytes_deserialize",
        serialize_with = "bytes_serialize"
    )]
    pub default_checkpoint: Vec<u8>,
    pub chain: ChainConfig,
    pub forks: Forks,
    #[serde(default)]
    pub max_checkpoint_age: u64,
    #[serde(default)]
    pub data_dir: Option<PathBuf>,
    #[serde(default)]
    pub load_external_fallback: bool,
    #[serde(default)]
    pub strict_checkpoint_age: bool,
}

impl Default for BaseConfig {
    fn default() -> Self {
        BaseConfig {
            rpc_bind_ip: IpAddr::V4(Ipv4Addr::LOCALHOST), // Default to "127.0.0.1"
            rpc_port: 0,
            consensus_rpc: None,
            default_checkpoint: vec![],
            chain: Default::default(),
            forks: Default::default(),
            max_checkpoint_age: 0,
            data_dir: None,
            load_external_fallback: false,
            strict_checkpoint_age: false,
        }
    }
}

impl BaseConfig {
    pub fn from_yaml_file(relative_path: &str) -> Result<Self, Report> {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push(relative_path);

        let file_content = fs::read_to_string(path)?;
        let mut config: BaseConfig = serde_yaml::from_str(&file_content)?;

        if config.max_checkpoint_age == 0 {
            config.max_checkpoint_age = 1_209_600; // 14 days
        }
        Ok(config)
    }
}
