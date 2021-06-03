use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::{HumanAddr, Uint128};
use cosmwasm_bignumber::{Uint256};
use cw20::Cw20ReceiveMsg;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {
    /// Owner address for config update
    pub owner_addr: HumanAddr,
    /// stable coin denom used to borrow & repay
    pub stable_denom: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {    
    ////////////////////
    /// Owner operations
    ////////////////////
    /// Register Contracts contract address
    RegisterContracts {
        market_contract: HumanAddr,
        aterra_contract: HumanAddr,
        cterra_contract: HumanAddr,
        capacorp_contract: HumanAddr,
        capa_contract: HumanAddr,
        insurance_contract: HumanAddr,           
    },
    /// Update config values
    UpdateConfig {
        owner_addr: Option<HumanAddr>,
    },
    Distribute {
        owner_addr: Option<HumanAddr>,
    },
    ////////////////////
    /// User operations
    ////////////////////
    /// Deposit stable asset to get interest
    Deposit {},
    Receive(Cw20ReceiveMsg),
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum DepositStableHandleMsg {
    DepositStable {},
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RedeemStableHookMsg {
    RedeemStable {},
}
        
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
}


// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub owner_addr: HumanAddr,
    pub market_contract: HumanAddr,
    pub aterra_contract: HumanAddr,
    pub cterra_contract: HumanAddr,
    pub capacorp_contract: HumanAddr,
    pub capa_contract: HumanAddr,
    pub insurance_contract: HumanAddr,
    pub stable_denom: String,
}
