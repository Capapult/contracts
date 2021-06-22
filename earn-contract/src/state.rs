use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{CanonicalAddr, Storage, StdResult, ReadonlyStorage};
use cosmwasm_storage::{ReadonlySingleton, Singleton, ReadonlyPrefixedStorage};
use cosmwasm_bignumber::{Uint256};

pub static KEY_CONFIG: &[u8] = b"config";
pub const KEY_STATE: &[u8] = b"state";
pub const KEY_BALANCE: &[u8] = b"balance";
const PREFIX_PROFIT: &[u8] = b"profit";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub contract_addr: CanonicalAddr,
    pub owner_addr: CanonicalAddr,
    pub market_contract: CanonicalAddr,
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

pub fn balances_prefix_read<S: ReadonlyStorage>(storage: &S) -> ReadonlyPrefixedStorage<S> {
    ReadonlyPrefixedStorage::new(KEY_BALANCE, storage)
}
pub fn store_profit<S: Storage>(storage: &mut S, profit: &Uint256) -> StdResult<()>  {
    Singleton::new(storage, PREFIX_PROFIT).save(profit)
}

pub fn read_profit<S: ReadonlyStorage>(storage: &S) -> StdResult<Uint256> {
    ReadonlySingleton::new(storage, PREFIX_PROFIT).load()
}