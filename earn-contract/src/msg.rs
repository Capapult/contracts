use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::{Addr, Uint128};
use cw20::Cw20ReceiveMsg;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    /// Owner address for config update
    pub owner_addr: String,
    /// stable coin denom used to borrow & repay
    pub stable_denom: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    ////////////////////
    /// Owner operations
    ////////////////////
    /// Register Contracts contract address
    RegisterContracts {
        market_contract: String,
        aterra_contract: String,
        cterra_contract: String,
        capacorp_contract: String,
        capa_contract: String,
        insurance_contract: String,
    },
    /// Update config values
    UpdateConfig {
        owner_addr: Option<Addr>,
    },
    Distribute {},
    //  Fees {},
    ////////////////////
    /// User operations
    ////////////////////
    /// Deposit stable asset to get interest
    Deposit {},
    Receive(Cw20ReceiveMsg),
    Harvest {},
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
    ExchangeRate {},
    Dashboard {},
    CorpAccounts {},
    HarvestValue { account_addr: String },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryStateMsg {
    State {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Account {
    AllAccounts {},
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub owner_addr: String,
    pub market_contract: String,
    pub aterra_contract: String,
    pub cterra_contract: String,
    pub capacorp_contract: String,
    pub capa_contract: String,
    pub insurance_contract: String,
    pub stable_denom: String,
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MarketStateResponse {
    pub total_liabilities: Decimal256,
    pub total_reserves: Decimal256,
    pub last_interest_updated: u64,
    pub last_reward_updated: u64,
    pub global_interest_index: Decimal256,
    pub global_reward_index: Decimal256,
    pub anc_emission_rate: Decimal256,
    pub prev_aterra_supply: Uint256,
    pub prev_exchange_rate: Decimal256,
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct DashboardResponse {
    pub total_value_locked: Uint256,
    pub cust_total_supply: Uint256,
    pub cust_nb_accounts: Uint256,
    pub cust_avg_balance: Decimal256,
    pub current_profit: Uint256,
    pub total_profit: Uint256,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct TokenInfoResponse {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub total_supply: Uint128,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug, Default)]
pub struct AllAccountsResponse {
    pub accounts: Vec<Addr>,
}
