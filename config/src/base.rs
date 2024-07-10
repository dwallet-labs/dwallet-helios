use std::{
    default::Default,
    fs,
    net::{IpAddr, Ipv4Addr},
    path::PathBuf,
};

use serde::{Deserialize, Serialize};

use crate::{
    types::{ChainConfig, Forks},
    utils::{bytes_deserialize, bytes_serialize},
};

const SUI_DIR: &str = ".dwallet";
const SUI_CONFIG_DIR: &str = "dwallet_config";
const ETH_LOCAL_NETWORK_CONFIG: &str = "eth_config.yaml";

/// The base configuration for a network.
#[derive(Serialize, Deserialize)]
pub struct BaseConfig {
    #[serde(default = "default_ipv4")]
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
    pub fn from_yaml_file() -> anyhow::Result<Self> {
        let mut path = sui_config_dir()?;
        path.push(ETH_LOCAL_NETWORK_CONFIG);

        let file_content = fs::read_to_string(path)?;
        let mut config: BaseConfig = serde_yaml::from_str(&file_content)?;

        if config.max_checkpoint_age == 0 {
            config.max_checkpoint_age = 1_209_600; // 14 days
        }
        Ok(config)
    }
}

fn default_ipv4() -> IpAddr {
    IpAddr::V4(Ipv4Addr::LOCALHOST)
}

pub fn sui_config_dir() -> anyhow::Result<PathBuf> {
    match std::env::var_os("SUI_CONFIG_DIR") {
        Some(config_env) => Ok(config_env.into()),
        None => match dirs::home_dir() {
            Some(v) => Ok(v.join(SUI_DIR).join(SUI_CONFIG_DIR)),
            None => anyhow::bail!("Cannot obtain home directory path"),
        },
    }
    .and_then(|dir| {
        if !dir.exists() {
            fs::create_dir_all(dir.clone())?;
        }
        Ok(dir)
    })
}
