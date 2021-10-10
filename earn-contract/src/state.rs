use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_bignumber::Uint256;
use cosmwasm_std::{StdResult, Storage};
use cosmwasm_storage::{ReadonlyPrefixedStorage, ReadonlySingleton, Singleton};

pub static KEY_CONFIG: &[u8] = b"config";
pub const KEY_STATE: &[u8] = b"state";
pub const KEY_BALANCE: &[u8] = b"balance";
const PREFIX_PROFIT: &[u8] = b"profit";
const PREFIX_TOTAL_DEPOSIT: &str = "td_";
const PREFIX_TOTAL_CLAIM: &str = "tc_";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub contract_addr: String,
    pub owner_addr: String,
    pub market_contract: String,
    pub aterra_contract: String,
    pub cterra_contract: String,
    pub capacorp_contract: String,
    pub capa_contract: String,
    pub insurance_contract: String,
    pub stable_denom: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {}

pub fn store_config(storage: &mut dyn Storage, data: &Config) -> StdResult<()> {
    Singleton::new(storage, KEY_CONFIG).save(data)
}

pub fn read_config(storage: &dyn Storage) -> StdResult<Config> {
    ReadonlySingleton::new(storage, KEY_CONFIG).load()
}

pub fn store_state(storage: &mut dyn Storage, data: &State) -> StdResult<()> {
    Singleton::new(storage, KEY_STATE).save(data)
}

pub fn read_state(storage: &dyn Storage) -> StdResult<State> {
    ReadonlySingleton::new(storage, KEY_STATE).load()
}

pub fn balances_prefix_read(storage: &dyn Storage) -> ReadonlyPrefixedStorage {
    ReadonlyPrefixedStorage::new(storage, KEY_BALANCE)
}
pub fn store_profit(storage: &mut dyn Storage, profit: &Uint256) -> StdResult<()> {
    Singleton::new(storage, PREFIX_PROFIT).save(profit)
}

pub fn read_profit(storage: &dyn Storage) -> StdResult<Uint256> {
    ReadonlySingleton::new(storage, PREFIX_PROFIT).load()
}

fn get_total_deposit_key(account_addr: String) -> String {
    let mut str_key: String = String::from(PREFIX_TOTAL_DEPOSIT);
    str_key.push_str(account_addr.as_str());
    str_key
}

pub fn store_total_deposit(
    storage: &mut dyn Storage,
    account_addr: &str,
    deposit: &Uint256,
) -> StdResult<()> {
    let str_key = get_total_deposit_key(String::from(account_addr));
    let key: &[u8] = str_key.as_bytes();
    Singleton::new(storage, key).save(deposit)
}

pub fn read_total_deposit(storage: &dyn Storage, account_addr: &str) -> Uint256 {
    let str_key = get_total_deposit_key(String::from(account_addr));
    let key: &[u8] = str_key.as_bytes();
    let res = ReadonlySingleton::new(storage, key).load();
    let mut current_deposit = Uint256::from(0u128);
    match res {
        Ok(x) => current_deposit = x,
        Err(_x) => {}
    }
    current_deposit
}

fn get_total_claim_key(account_addr: String) -> String {
    let mut str_key: String = String::from(PREFIX_TOTAL_CLAIM);
    str_key.push_str(account_addr.as_str());
    str_key
}

pub fn store_total_claim(
    storage: &mut dyn Storage,
    account_addr: &str,
    deposit: &Uint256,
) -> StdResult<()> {
    let str_key = get_total_claim_key(String::from(account_addr));
    let key: &[u8] = str_key.as_bytes();
    Singleton::new(storage, key).save(deposit)
}

pub fn read_total_claim(storage: &dyn Storage, account_addr: &str) -> Uint256 {
    let str_key = get_total_claim_key(String::from(account_addr));
    let key: &[u8] = str_key.as_bytes();
    let res = ReadonlySingleton::new(storage, key).load();
    let mut current_claim = Uint256::from(0u128);
    match res {
        Ok(x) => current_claim = x,
        Err(_x) => {}
    }
    current_claim
}
