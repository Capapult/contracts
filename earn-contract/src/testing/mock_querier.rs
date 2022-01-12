use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::str;

use crate::msg::{ConfigResponse, MarketStateResponse};
use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::testing::{MockApi, MockQuerier, MockStorage, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{
    from_binary, from_slice, to_binary, Coin, ContractResult, Decimal, OwnedDeps, Querier,
    QuerierResult, QueryRequest, SystemError, SystemResult, Uint128, WasmQuery,
};
use cosmwasm_storage::to_length_prefixed;
use cw20::{AllAccountsResponse, TokenInfoResponse, BalanceResponse};
use std::collections::HashMap;
use terra_cosmwasm::{TaxCapResponse, TaxRateResponse, TerraQuery, TerraQueryWrapper, TerraRoute};

// github link for col5 update https://github.com/Anchor-Protocol/money-market-contracts/commits/main/contracts/market/src/testing/mock_querier.rs

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// Query overseer config to get target deposit rate
    Config {},
    State {},
    /// Query cw20 Token Info
    TokenInfo {},
    Balance {
        address: String,
    },
    AllAccounts {},
}
/// mock_dependencies is a drop-in replacement for cosmwasm_std::testing::mock_dependencies
/// this uses our CustomQuerier.
pub fn mock_dependencies(
    contract_balance: &[Coin],
) -> OwnedDeps<MockStorage, MockApi, WasmMockQuerier> {
    let custom_querier: WasmMockQuerier =
        WasmMockQuerier::new(MockQuerier::new(&[(MOCK_CONTRACT_ADDR, contract_balance)]));

    OwnedDeps {
        storage: MockStorage::default(),
        api: MockApi::default(),
        querier: custom_querier,
    }
}

pub struct WasmMockQuerier {
    base: MockQuerier<TerraQueryWrapper>,
    token_querier: TokenQuerier,
    tax_querier: TaxQuerier,
}

#[derive(Clone, Default)]
pub struct TokenQuerier {
    // this lets us iterate over all pairs that match the first string
    balances: HashMap<String, HashMap<String, Uint128>>,
}

impl TokenQuerier {
    pub fn new(balances: &[(&String, &[(&String, &Uint128)])]) -> Self {
        TokenQuerier {
            balances: balances_to_map(balances),
        }
    }
}

pub(crate) fn balances_to_map(
    balances: &[(&String, &[(&String, &Uint128)])],
) -> HashMap<String, HashMap<String, Uint128>> {
    let mut balances_map: HashMap<String, HashMap<String, Uint128>> = HashMap::new();
    for (contract_addr, balances) in balances.iter() {
        let mut contract_balances_map: HashMap<String, Uint128> = HashMap::new();
        for (addr, balance) in balances.iter() {
            contract_balances_map.insert(String::from(*addr), **balance);
        }

        balances_map.insert(String::from(*contract_addr), contract_balances_map);
        println!(
            "balances_map.insert contract_addr {}",
            String::from(*contract_addr)
        );
    }
    balances_map
}

#[derive(Clone, Default)]
pub struct TaxQuerier {
    rate: Decimal,
    // this lets us iterate over all pairs that match the first string
    caps: HashMap<String, Uint128>,
}

impl TaxQuerier {
    pub fn new(rate: Decimal, caps: &[(&String, &Uint128)]) -> Self {
        TaxQuerier {
            rate,
            caps: caps_to_map(caps),
        }
    }
}
pub(crate) fn caps_to_map(caps: &[(&String, &Uint128)]) -> HashMap<String, Uint128> {
    let mut owner_map: HashMap<String, Uint128> = HashMap::new();
    for (denom, cap) in caps.iter() {
        owner_map.insert(denom.to_string(), **cap);
    }
    owner_map
}

impl Querier for WasmMockQuerier {
    fn raw_query(&self, bin_request: &[u8]) -> QuerierResult {
        // MockQuerier doesn't support Custom, so we ignore it completely here
        let request: QueryRequest<TerraQueryWrapper> = match from_slice(bin_request) {
            Ok(v) => v,
            Err(e) => {
                return SystemResult::Err(SystemError::InvalidRequest {
                    error: format!("Parsing query request: {}", e),
                    request: bin_request.into(),
                })
            }
        };
        self.handle_query(&request)
    }
}

impl WasmMockQuerier {
    pub fn handle_query(&self, request: &QueryRequest<TerraQueryWrapper>) -> QuerierResult {
        match &request {
            QueryRequest::Custom(TerraQueryWrapper { route, query_data }) => {
                if &TerraRoute::Treasury == route {
                    match query_data {
                        TerraQuery::TaxRate {} => {
                            let res = TaxRateResponse {
                                rate: self.tax_querier.rate,
                            };
                            SystemResult::Ok(ContractResult::from(to_binary(&res)))
                        }
                        TerraQuery::TaxCap { denom } => {
                            let cap = self
                                .tax_querier
                                .caps
                                .get(denom)
                                .copied()
                                .unwrap_or_default();
                            let res = TaxCapResponse { cap };
                            SystemResult::Ok(ContractResult::from(to_binary(&res)))
                        }
                        _ => panic!("DO NOT ENTER HERE"),
                    }
                } else {
                    panic!("DO NOT ENTER HERE")
                }
            }
            QueryRequest::Wasm(WasmQuery::Smart { contract_addr, msg }) => {

                let matcher : QueryMsg = from_binary(&msg).unwrap();
                match matcher {
                    QueryMsg::AllAccounts {} => {
                        let mut vec = Vec::new();
                        vec.push("daniel".to_string());
                        vec.push("bruno".to_string());

                        SystemResult::Ok(ContractResult::from(to_binary(&AllAccountsResponse {
                            accounts: vec,
                        })))
                    }
                    QueryMsg::Config {} => {
                        SystemResult::Ok(ContractResult::from(to_binary(&ConfigResponse {
                            owner_addr: String::from(""),
                            aterra_contract: String::from(""),
                            market_contract: String::from(""),
                            cterra_contract: String::from(""),
                            capa_contract: String::from(""),
                            capacorp_contract: String::from(""),
                            insurance_contract: String::from(""),
                            stable_denom: "uusd".to_string(),
                        })))
                    }
                    QueryMsg::State {} => {
                        SystemResult::Ok(ContractResult::from(to_binary(&MarketStateResponse {
                            total_liabilities: Decimal256::zero(),
                            total_reserves: Decimal256::zero(),
                            last_interest_updated: 0,
                            last_reward_updated: 0,
                            global_interest_index: Decimal256::one(),
                            global_reward_index: Decimal256::zero(),
                            anc_emission_rate: Decimal256::zero(),
                            prev_aterra_supply: Uint256::zero(),
                            prev_exchange_rate: Decimal256::one(),
                        })))
                    }
                    QueryMsg::Balance { address } => {
                        let balances: HashMap<String, Uint128> =
                            match self.token_querier.balances.get(contract_addr) {
                                Some(balances) => balances.clone(),
                                None => HashMap::new(),
                            };
                        let option_balance = balances.get(&address);

                        let mut balance : &Uint128 = &Uint128::zero();
                        if option_balance.is_none() == false {
                            balance = option_balance.unwrap();
                        }

                        SystemResult::Ok(ContractResult::from(to_binary(&BalanceResponse { 
                            balance: *balance
                        })))
                    }
                    QueryMsg::TokenInfo {} => {
                        let balances: HashMap<String, Uint128> =
                            match self.token_querier.balances.get(contract_addr) {
                                Some(balances) => balances.clone(),
                                None => HashMap::new(),
                            };

                        let mut total_supply = Uint128::zero();

                        for balance in balances {
                            total_supply += balance.1;
                        }

                        SystemResult::Ok(ContractResult::from(to_binary(&TokenInfoResponse {
                            name: "mAPPL".to_string(),
                            symbol: "mAPPL".to_string(),
                            decimals: 6,
                            total_supply,
                        })))
                    }
                }
            }
            QueryRequest::Wasm(WasmQuery::Raw { contract_addr, key }) => {
                let key: &[u8] = key.as_slice();
                let address: &str = str::from_utf8(key).unwrap();

                let prefix_token_info = to_length_prefixed(b"token_info").to_vec();
                let prefix_balance = to_length_prefixed(b"balance").to_vec();

                let balances: HashMap<String, Uint128> =
                    match self.token_querier.balances.get(contract_addr) {
                        Some(balances) => balances.clone(),
                        None => HashMap::new(),
                    };

                if key.to_vec() == prefix_token_info {
                    let mut total_supply = Uint128::zero();

                    for balance in balances {
                        total_supply += balance.1;
                    }

                    SystemResult::Ok(ContractResult::from(to_binary(&TokenInfoResponse {
                        name: "mAPPL".to_string(),
                        symbol: "mAPPL".to_string(),
                        decimals: 6,
                        total_supply: total_supply,
                    })))
                } else if key[..prefix_balance.len()].to_vec() == prefix_balance {
                    let key_address: &[u8] = &key[prefix_balance.len()..];
                    let address: &str = str::from_utf8(key_address).unwrap();

                    let balance = match balances.get(address) {
                        Some(v) => v,
                        None => {
                            return SystemResult::Err(SystemError::InvalidRequest {
                                error: "Balance not found".to_string(),
                                request: key.into(),
                            });
                        }
                    };

                    SystemResult::Ok(ContractResult::from(to_binary(&BalanceResponse { 
                        balance: *balance
                    })))
                } else {
                    panic!("DO NOT ENTER HERE")
                }
            }
            _ => self.base.handle_query(request),
        }
    }
}

impl WasmMockQuerier {
    pub fn new(base: MockQuerier<TerraQueryWrapper>) -> Self {
        WasmMockQuerier {
            base,
            token_querier: TokenQuerier::default(),
            tax_querier: TaxQuerier::default(),
        }
    }

    // set a new balance for the given address and return the old balance
    pub fn update_balance<U: Into<String>>(
        &mut self,
        addr: U,
        balance: Vec<Coin>,
    ) -> Option<Vec<Coin>> {
        println!("update_balance");
        self.base.update_balance(addr, balance)
    }

    // configure the mint whitelist mock querier
    pub fn with_token_balances(&mut self, balances: &[(&String, &[(&String, &Uint128)])]) {
        self.token_querier = TokenQuerier::new(balances);
    }
}
