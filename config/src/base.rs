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
    CHECKPOINT_AGE_14_DAYS,
};

const SUI_DIR: &str = ".dwallet";
const SUI_CONFIG_DIR: &str = "dwallet_config";
const ETH_LOCAL_NETWORK_CONFIG: &str = "eth_config.yaml";
const ENV_SUI_CONFIG_DIR: &str = "SUI_CONFIG_DIR";

/// The base configuration for a network.
#[derive(Serialize, Deserialize)]
#[cfg_attr(test, derive(Debug))]
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
            // Default to "127.0.0.1".
            rpc_bind_ip: IpAddr::V4(Ipv4Addr::LOCALHOST),
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
    /// Load local network configuration from Yaml file.
    pub fn from_yaml_file() -> anyhow::Result<Self> {
        let path = sui_config_dir()?.join(ETH_LOCAL_NETWORK_CONFIG);

        let file_content = fs::read_to_string(path)?;
        let mut config: BaseConfig = serde_yaml::from_str(&file_content)?;

        if config.max_checkpoint_age == 0 {
            config.max_checkpoint_age = CHECKPOINT_AGE_14_DAYS;
        }
        Ok(config)
    }
}

fn default_ipv4() -> IpAddr {
    IpAddr::V4(Ipv4Addr::LOCALHOST)
}

/// Get the Sui config directory.
/// If the directory does not exist, it will be created.
pub fn sui_config_dir() -> anyhow::Result<PathBuf> {
    std::env::var_os(ENV_SUI_CONFIG_DIR)
        .map(Into::into)
        .or_else(|| dirs::home_dir().map(|home| home.join(SUI_DIR).join(SUI_CONFIG_DIR)))
        .ok_or_else(|| anyhow::anyhow!("cannot get the home directory path"))
        .and_then(|dir| {
            if !dir.exists() {
                fs::create_dir_all(&dir)?;
            }
            Ok(dir)
        })
}

#[cfg(test)]
mod test {
    use std::{env, fs, io::Write, net::Ipv4Addr, path::PathBuf};

    use common::utils::hex_str_to_bytes;
    use tempdir::TempDir;

    use super::*;

    #[test]
    fn test_config_from_yaml_success() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests");
        unsafe {
            env::set_var(ENV_SUI_CONFIG_DIR, path.to_str().unwrap());
        }

        let base_config = BaseConfig::from_yaml_file().unwrap();

        assert_eq!(
            Some("http://localhost:3500".to_string()),
            base_config.consensus_rpc,
        );

        assert_eq!(8545, base_config.rpc_port);
        assert_eq!(Ipv4Addr::new(0, 0, 0, 0), base_config.rpc_bind_ip);
        assert_eq!(
            hex_str_to_bytes("0xb3b5d384704782ab9e17969ad692340b1f8952baf5e8089b8317fc45f6b360e3")
                .unwrap(),
            base_config.default_checkpoint
        );
        assert_eq!(32382, base_config.chain.chain_id);
        assert_eq!(1720442031, base_config.chain.genesis_time);
        assert_eq!(
            hex_str_to_bytes("0x83431ec7fcf92cfc44947fc0418e831c25e1d0806590231c439830db7ad54fda")
                .unwrap(),
            base_config.chain.genesis_root
        );
        assert_eq!(0, base_config.forks.genesis.epoch);
        assert_eq!(
            hex_str_to_bytes("0x20000089").unwrap(),
            base_config.forks.genesis.fork_version
        );
        assert_eq!(0, base_config.forks.altair.epoch);
        assert_eq!(
            hex_str_to_bytes("0x20000090").unwrap(),
            base_config.forks.altair.fork_version
        );
        assert_eq!(0, base_config.forks.bellatrix.epoch);
        assert_eq!(
            hex_str_to_bytes("0x20000091").unwrap(),
            base_config.forks.bellatrix.fork_version
        );
        assert_eq!(0, base_config.forks.capella.epoch);
        assert_eq!(
            hex_str_to_bytes("0x20000092").unwrap(),
            base_config.forks.capella.fork_version
        );
        assert_eq!(132608, base_config.forks.deneb.epoch);
        assert_eq!(
            hex_str_to_bytes("0x20000093").unwrap(),
            base_config.forks.deneb.fork_version
        );
    }

    #[test]
    fn test_config_from_yaml_fail() {
        // Load config file manually.
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join(ETH_LOCAL_NETWORK_CONFIG);

        // Read file and replace chain_id with invalid value.
        let file_content = fs::read_to_string(path).unwrap();
        let modified_file = file_content.replace("chain_id: 32382", "chain_id: ~");

        // Create a temporary directory and write the modified file to it.
        let temp_dir = TempDir::new("dwallet_config").unwrap();
        let path_to_broken_config = temp_dir.path().join(ETH_LOCAL_NETWORK_CONFIG);
        let mut broken_config_file = fs::File::create(&path_to_broken_config).unwrap();
        broken_config_file
            .write_all(modified_file.as_bytes())
            .unwrap();

        // Set env variable to the temporary directory.
        unsafe {
            env::set_var(ENV_SUI_CONFIG_DIR, temp_dir.path());
        }
        let base_config = BaseConfig::from_yaml_file();
        assert_eq!(
            "chain.chain_id: invalid type: unit value, expected u64 at line 7 column 13",
            base_config.unwrap_err().to_string(),
        );

        drop(broken_config_file);
        temp_dir.close().unwrap();
    }
}
