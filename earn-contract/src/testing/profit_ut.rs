use crate::contract::{handle, init, INITIAL_DEPOSIT_AMOUNT};
use crate::msg::{HandleMsg, InitMsg, RedeemStableHookMsg};
use crate::state::Config;

use crate::testing::mock_querier::mock_dependencies;
use cosmwasm_std::testing::{mock_env, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{from_binary, log, to_binary, Api, Coin, HumanAddr, StdError, Uint128};

#[test]
fn proper_calculate_fees() {
}

#[test]
fn proper_calculate_profit() {
}

#[test]
fn proper_transfer_capacorp() {
}

#[test]
fn proper_distribute() {
}