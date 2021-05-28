use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::{CanonicalAddr, Storage, StdResult};
use cosmwasm_storage::{bucket, bucket_read, ReadonlyBucket, ReadonlySingleton, Singleton};

pub static KEY_CONFIG: &[u8] = b"config";
pub const KEY_STATE: &[u8] = b"state";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub contract_addr: CanonicalAddr,
    pub owner_addr: CanonicalAddr,
    pub aterra_contract: CanonicalAddr,
    pub cterra_contract: CanonicalAddr,
    pub capacorp_contract: CanonicalAddr,
    pub capa_contract: CanonicalAddr,
    pub insurance_contract: CanonicalAddr,
    pub stable_denom: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
}

pub fn store_config<S: Storage>(storage: &mut S, data: &Config) -> StdResult<()> {
    Singleton::new(storage, KEY_CONFIG).save(data)
}

pub fn read_config<S: Storage>(storage: &S) -> StdResult<Config> {
    ReadonlySingleton::new(storage, KEY_CONFIG).load()
}

pub fn store_state<S: Storage>(storage: &mut S, data: &State) -> StdResult<()> {
    Singleton::new(storage, KEY_STATE).save(data)
}

pub fn read_state<S: Storage>(storage: &S) -> StdResult<State> {
    ReadonlySingleton::new(storage, KEY_STATE).load()
}