use std::{collections::HashMap, fmt};

pub use ethers::types::Address;
use ethers::types::{Bytes, H256, U256};
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone)]
pub struct Account {
    pub balance: U256,
    pub nonce: u64,
    pub code_hash: H256,
    pub code: Vec<u8>,
    pub storage_hash: H256,
    // `slots` type changed to HashMap<U256, U256> from HashMap<H256, H256> because of a bugfix in
    // `ethers` crate. The bugfix is in the `ethers` crate version 2.0.14.
    pub slots: HashMap<U256, U256>,
}

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CallOpts {
    pub from: Option<Address>,
    pub to: Option<Address>,
    pub gas: Option<U256>,
    pub gas_price: Option<U256>,
    pub value: Option<U256>,
    pub data: Option<Bytes>,
}

impl fmt::Debug for CallOpts {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CallOpts")
            .field("from", &self.from)
            .field("to", &self.to)
            .field("value", &self.value)
            .field("data", &hex::encode(self.data.clone().unwrap_or_default()))
            .finish()
    }
}
