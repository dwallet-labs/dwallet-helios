use std::{env, fs, io::Write, net::Ipv4Addr, path::PathBuf};

use common::utils::hex_str_to_bytes;
use config::BaseConfig;
use tempdir::TempDir;

#[test]
fn test_config_from_yaml_success() {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests");

    env::set_var("SUI_CONFIG_DIR", path.to_str().unwrap());

    let base_config = BaseConfig::from_yaml_file().unwrap();

    assert_eq!(
        base_config.consensus_rpc,
        Some("http://localhost:3500".to_string())
    );
    assert_eq!(base_config.rpc_port, 8545);
    assert_eq!(base_config.rpc_bind_ip, Ipv4Addr::new(0, 0, 0, 0));
    assert_eq!(
        base_config.default_checkpoint,
        hex_str_to_bytes("0xb3b5d384704782ab9e17969ad692340b1f8952baf5e8089b8317fc45f6b360e3")
            .unwrap()
    );
    assert_eq!(base_config.chain.chain_id, 32382);
    assert_eq!(base_config.chain.genesis_time, 1720442031);
    assert_eq!(
        base_config.chain.genesis_root,
        hex_str_to_bytes("0x83431ec7fcf92cfc44947fc0418e831c25e1d0806590231c439830db7ad54fda")
            .unwrap()
    );
    assert_eq!(base_config.forks.genesis.epoch, 0);
    assert_eq!(
        base_config.forks.genesis.fork_version,
        hex_str_to_bytes("0x20000089").unwrap()
    );
    assert_eq!(base_config.forks.altair.epoch, 0);
    assert_eq!(
        base_config.forks.altair.fork_version,
        hex_str_to_bytes("0x20000090").unwrap()
    );
    assert_eq!(base_config.forks.bellatrix.epoch, 0);
    assert_eq!(
        base_config.forks.bellatrix.fork_version,
        hex_str_to_bytes("0x20000091").unwrap()
    );
    assert_eq!(base_config.forks.capella.epoch, 0);
    assert_eq!(
        base_config.forks.capella.fork_version,
        hex_str_to_bytes("0x20000092").unwrap()
    );
    assert_eq!(base_config.forks.deneb.epoch, 132608);
    assert_eq!(
        base_config.forks.deneb.fork_version,
        hex_str_to_bytes("0x20000093").unwrap()
    );
}

#[test]
fn test_config_from_yaml_fail() {
    // Load config file manually.
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests");
    path.push("eth_config.yaml");

    // Read file and replace chain_id with invalid value.
    let file_content = fs::read_to_string(path).unwrap();
    let file_content = file_content.replace("chain_id: 82385", "chain_id: ~");

    // Create a temporary directory and write the modified file to it.
    let temp_dir = TempDir::new("dwallet_config").unwrap();
    let path_to_broken_config = temp_dir.path().join("eth_config_broken.yaml");
    let mut broken_config_file = fs::File::create(&path_to_broken_config).unwrap();
    broken_config_file
        .write_all(file_content.as_bytes())
        .unwrap();

    // Set env variable to the temporary directory.
    env::set_var("SUI_CONFIG_DIR", temp_dir.path());
    let base_config = BaseConfig::from_yaml_file();

    assert!(base_config.is_err());

    drop(broken_config_file);
    temp_dir.close().unwrap();
}
