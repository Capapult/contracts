use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::HumanAddr;

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
    Distribute {},
    ////////////////////
    /// User operations
    ////////////////////
    /// Deposit stable asset to get interest
    Deposit {},
    Withdraw {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    // GetCount returns the current count as a json-encoded number
    GetCount {},
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CountResponse {
    pub count: i32,
}
