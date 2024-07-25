use std::cmp;

use async_trait::async_trait;
use common::errors::RpcError;
use eyre::Result;
use retri::{retry, BackoffSettings};
use serde::de::DeserializeOwned;
use crate::types::utils::header_deserialize;
use super::ConsensusRpc;
use crate::{constants::MAX_REQUEST_LIGHT_CLIENT_UPDATES, types::*};

#[derive(Debug)]
pub struct NimbusRpc {
    rpc: String,
}

async fn get<R: DeserializeOwned>(req: &str) -> Result<R> {
    let bytes = retry(
        || async { Ok::<_, eyre::Report>(reqwest::get(req).await?.bytes().await?) },
        BackoffSettings::default(),
    )
        .await?;

    Ok(serde_json::from_slice::<R>(&bytes)?)
}

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
impl ConsensusRpc for NimbusRpc {
    fn new(rpc: &str) -> Self {
        NimbusRpc {
            rpc: rpc.to_string(),
        }
    }

    async fn get_bootstrap(&self, block_root: &'_ [u8]) -> Result<Bootstrap> {
        let root_hex = hex::encode(block_root);
        let req = format!(
            "{}/eth/v1/beacon/light_client/bootstrap/0x{}",
            self.rpc, root_hex
        );

        let res: BootstrapResponse = get(&req).await.map_err(|e| RpcError::new("bootstrap", e))?;

        Ok(res.data)
    }

    async fn get_updates(&self, period: u64, count: u8) -> Result<Vec<Update>> {
        let count = cmp::min(count, MAX_REQUEST_LIGHT_CLIENT_UPDATES);
        let req = format!(
            "{}/eth/v1/beacon/light_client/updates?start_period={}&count={}",
            self.rpc, period, count
        );

        let res: UpdateResponse = get(&req).await.map_err(|e| RpcError::new("updates", e))?;

        Ok(res.into_iter().map(|d| d.data).collect())
    }

    async fn get_finality_update(&self) -> Result<FinalityUpdate> {
        let req = format!("{}/eth/v1/beacon/light_client/finality_update", self.rpc);
        let res: FinalityUpdateResponse = get(&req)
            .await
            .map_err(|e| RpcError::new("finality_update", e))?;

        Ok(res.data)
    }

    async fn get_optimistic_update(&self) -> Result<OptimisticUpdate> {
        let req = format!("{}/eth/v1/beacon/light_client/optimistic_update", self.rpc);
        let res: OptimisticUpdateResponse = get(&req)
            .await
            .map_err(|e| RpcError::new("optimistic_update", e))?;

        Ok(res.data)
    }

    async fn get_block(&self, slot: u64) -> Result<BeaconBlock> {
        let req = format!("{}/eth/v2/beacon/blocks/{}", self.rpc, slot);
        let res: BeaconBlockResponse = get(&req).await.map_err(|e| RpcError::new("blocks", e))?;

        Ok(res.data.message)
    }

    async fn get_block_header(&self, slot: u64) -> Result<Header> {
        // let req = format!("{}/eth/v1/beacon/headers{}", self.rpc, slot);
        // let res: BeaconBlockHeaderResponse = get(&req).await.map_err(|e| RpcError::new("blocks", e))?;
        // parse from json string
        let res = r#"{
"execution_optimistic": false,
"finalized": false,
"data": {
"root": "0xe314b68c0b2b4a492bf94139c4508abd183c196c6c2adb38f323ebab24a6aa35",
"canonical": true,
"header": {
"message": {
"slot": "9583479",
"proposer_index": "1137277",
"parent_root": "0x29d91e360e5bc177f0ee129d820b7db6af1fb9798e7f591ecea7acb50bbd1c12",
"state_root": "0x8539789689018cea692c491efa8876adf83b3af349243c6a818c6091c39bc3f5",
"body_root": "0xf5580fae842aa148b937daa74b673393e9758256322b25138200fde7b829b07f"
},
"signature": "0x976f3ffb933cb996b7e9b4e118108fe8404e150a490758ba55c95e787034161fd9702edebae6294cf095a797ad7b769e05864502e5e3431d753bf4a76d1df86a32fa04818a42fb8c69e2eea5762c1dd0d475e3c8b7c0524e8940d323e97020c8"
}
}
}"#;
        let res: BeaconBlockHeaderResponse = serde_json::from_str(res).unwrap();
        Ok(res.data.header.message)
    }

    async fn chain_id(&self) -> Result<u64> {
        let req = format!("{}/eth/v1/config/spec", self.rpc);
        let res: SpecResponse = get(&req).await.map_err(|e| RpcError::new("spec", e))?;

        Ok(res.data.chain_id.into())
    }
}

#[derive(serde::Deserialize, Debug)]
struct BeaconBlockResponse {
    data: BeaconBlockData,
}

#[derive(serde::Deserialize, Debug)]
struct BeaconBlockData {
    message: BeaconBlock,
}

#[derive(serde::Deserialize, Debug)]
struct BeaconBlockHeaderResponse {
    data: BeaconBlockHeaderData,
}

#[derive(serde::Deserialize, Debug)]
struct BeaconBlockHeaderData {
    header: HeaderMessage,
}

#[derive(serde::Deserialize, Debug)]
struct HeaderMessage {
    message: Header,
}

type UpdateResponse = Vec<UpdateData>;

#[derive(serde::Deserialize, Debug)]
struct UpdateData {
    data: Update,
}

#[derive(serde::Deserialize, Debug)]
struct FinalityUpdateResponse {
    data: FinalityUpdate,
}

#[derive(serde::Deserialize, Debug)]
struct OptimisticUpdateResponse {
    data: OptimisticUpdate,
}

#[derive(serde::Deserialize, Debug)]
struct BootstrapResponse {
    data: Bootstrap,
}

#[derive(serde::Deserialize, Debug)]
struct SpecResponse {
    data: Spec,
}

#[derive(serde::Deserialize, Debug)]
struct Spec {
    #[serde(rename = "DEPOSIT_NETWORK_ID")]
    chain_id: primitives::U64,
}
