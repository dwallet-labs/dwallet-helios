use std::{path::PathBuf, str::FromStr};

use ethers::{types::Address, utils};
use eyre::Result;
use tracing::info;
use tracing_subscriber::{
    filter::{EnvFilter, LevelFilter},
    FmtSubscriber,
};

use helios::{config::networks::Network, prelude::*};

#[tokio::main]
async fn main() -> Result<()> {
    let env_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env()
        .expect("invalid env filter");

    let subscriber = FmtSubscriber::builder()
        .with_env_filter(env_filter)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("subsriber set failed");

    let untrusted_rpc_url = "https://eth-mainnet.g.alchemy.com/v2/2_kOIKeqfT3_cU_P5bEHfUSwFVaI4geI";
    info!("Using untrusted RPC URL [REDACTED]");

    let consensus_rpc = "https://www.lightclientdata.org";
    info!("Using consensus RPC URL: {}", consensus_rpc);

    let mut client: Client<FileDB> = ClientBuilder::new()
        .network(Network::MAINNET)
        .consensus_rpc(consensus_rpc)
        .execution_rpc(untrusted_rpc_url)
        // .checkpoint("0x45a04ace82c71debcb49a27a9f4fa9e0be7d66c8fe2efacc9310d310c8237f55") // old
        .checkpoint("0x0520875f1dba863474f188c7bc1a380170b37f4bd4d523a3102be846af864d3d") // new
        .load_external_fallback()
        .data_dir(PathBuf::from("/tmp/helios"))
        .build()?;

    info!(
        "Built client on network \"{}\" with external checkpoint fallbacks",
        Network::MAINNET
    );

    // client.start().await?;
    client.wait_synced().await;

    let head_block_num = client.get_block_number().await?;
    let addr = Address::from_str("0x00000000219ab540356cBB839Cbe05303d7705Fa")?;
    let block = BlockTag::Latest;
    let balance = client.get_balance(&addr, block).await?;
    
    info!("synced up to block: {}", head_block_num);
    info!(
        "balance of deposit contract: {}",
        utils::format_ether(balance)
    );
    info!("done");

    Ok(())
}
