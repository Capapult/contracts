use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_bignumber::Uint256;
use cosmwasm_std::{Addr, CanonicalAddr, Order, StdResult, Storage};
use cosmwasm_storage::{Bucket, ReadonlyBucket, ReadonlySingleton, Singleton};
use cw_storage_plus::Map;

pub static KEY_CONFIG: &[u8] = b"config";
pub const KEY_BALANCE: &[u8] = b"balance";
const PREFIX_PROFIT: &[u8] = b"profit";
const PREFIX_TOTAL_DEPOSIT: &[u8] = b"td_";
const PREFIX_LAST_WITHDRAW: &[u8] = b"lw_";
const PREFIX_TOTAL_CLAIM: &[u8] = b"tc_";

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
    pub capa_yield: String,
}

pub fn store_config(storage: &mut dyn Storage, data: &Config) -> StdResult<()> {
    Singleton::new(storage, KEY_CONFIG).save(data)
}

pub fn read_config(storage: &dyn Storage) -> StdResult<Config> {
    ReadonlySingleton::new(storage, KEY_CONFIG).load()
}

pub fn store_profit(storage: &mut dyn Storage, profit: &Uint256) -> StdResult<()> {
    Singleton::new(storage, PREFIX_PROFIT).save(profit)
}

pub fn read_profit(storage: &dyn Storage) -> StdResult<Uint256> {
    ReadonlySingleton::new(storage, PREFIX_PROFIT).load()
}

pub fn store_total_deposit(
    storage: &mut dyn Storage,
    account_addr: &CanonicalAddr,
    deposit: &Uint256,
) -> StdResult<()> {
    Bucket::new(storage, PREFIX_TOTAL_DEPOSIT).save(account_addr.as_slice(), deposit)
}

pub fn read_total_deposit(storage: &dyn Storage, account_addr: &CanonicalAddr) -> Uint256 {
    let mut current_deposit = Uint256::from(0u128);
    if let Ok(x) = ReadonlyBucket::new(storage, PREFIX_TOTAL_DEPOSIT).load(account_addr.as_slice())
    {
        current_deposit = x
    }
    current_deposit
}

pub fn store_last_ops_ust(
    storage: &mut dyn Storage,
    account_addr: &CanonicalAddr,
    deposit: &Uint256,
) -> StdResult<()> {
    Bucket::new(storage, PREFIX_LAST_WITHDRAW).save(account_addr.as_slice(), deposit)
}

pub fn read_last_ops_ust(
    storage: &dyn Storage,
    account_addr: &CanonicalAddr,
    total_deposit: Uint256,
) -> Uint256 {
    let mut current_claim = total_deposit;
    if let Ok(x) = ReadonlyBucket::new(storage, PREFIX_LAST_WITHDRAW).load(account_addr.as_slice())
    {
        current_claim = x
    }
    current_claim
}

pub fn store_total_claim(
    storage: &mut dyn Storage,
    account_addr: &CanonicalAddr,
    deposit: &Uint256,
) -> StdResult<()> {
    Bucket::new(storage, PREFIX_TOTAL_CLAIM).save(account_addr.as_slice(), deposit)
}

pub fn read_total_claim(storage: &dyn Storage, account_addr: &CanonicalAddr) -> Uint256 {
    let mut total_claim = Uint256::from(0u128);
    if let Ok(x) = ReadonlyBucket::new(storage, PREFIX_TOTAL_CLAIM).load(account_addr.as_slice()) {
        total_claim = x
    }
    total_claim
}
pub fn remove_account(storage: &mut dyn Storage, account_addr: &CanonicalAddr) {
    Bucket::<Uint256>::new(storage, PREFIX_TOTAL_DEPOSIT).remove(account_addr.as_slice());
    Bucket::<Uint256>::new(storage, PREFIX_TOTAL_CLAIM).remove(account_addr.as_slice());
    Bucket::<Uint256>::new(storage, PREFIX_LAST_WITHDRAW).remove(account_addr.as_slice());
}
