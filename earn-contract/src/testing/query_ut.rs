use crate::contract::{execute, instantiate, query, INITIAL_DEPOSIT_AMOUNT};
use crate::msg::{ConfigResponse, DashboardResponse, ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::Config;
use crate::testing::mock_querier::{mock_dependencies, WasmMockQuerier};
use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::testing::{
    mock_env, mock_info, MockApi, MockQuerier, MockStorage, MOCK_CONTRACT_ADDR,
};
use cosmwasm_std::{
    attr, from_binary, to_binary, Api, Coin, Deps, DepsMut, MemoryStorage, OwnedDeps, StdError,
    StdResult, Uint128,
};

fn get_register_contracts(
    deps: &OwnedDeps<MockStorage, MockApi, WasmMockQuerier>,
    config: &Config,
) -> ExecuteMsg {
    ExecuteMsg::RegisterContracts {
        market_contract: deps
            .api
            .addr_humanize(&config.market_contract)
            .unwrap()
            .to_string(),
        aterra_contract: deps
            .api
            .addr_humanize(&config.aterra_contract)
            .unwrap()
            .to_string(),
        cterra_contract: deps
            .api
            .addr_humanize(&config.cterra_contract)
            .unwrap()
            .to_string(),
        capacorp_contract: deps
            .api
            .addr_humanize(&config.capacorp_contract)
            .unwrap()
            .to_string(),
        capa_contract: deps
            .api
            .addr_humanize(&config.capa_contract)
            .unwrap()
            .to_string(),
        insurance_contract: deps
            .api
            .addr_humanize(&config.insurance_contract)
            .unwrap()
            .to_string(),
    }
}

fn get_mock_config(deps: &OwnedDeps<MockStorage, MockApi, WasmMockQuerier>) -> Config {
    Config {
        contract_addr: deps.api.addr_canonicalize(MOCK_CONTRACT_ADDR).unwrap(),
        owner_addr: deps.api.addr_canonicalize("owner").unwrap(),
        aterra_contract: deps.api.addr_canonicalize("aterra_contract").unwrap(),
        market_contract: deps.api.addr_canonicalize("market").unwrap(),
        cterra_contract: deps.api.addr_canonicalize("cterra_contract").unwrap(),
        capacorp_contract: deps.api.addr_canonicalize("capacorp_contract").unwrap(),
        capa_contract: deps.api.addr_canonicalize("capa_contract").unwrap(),
        insurance_contract: deps.api.addr_canonicalize("insurance_contract").unwrap(),
        stable_denom: "uusd".to_string(),
        capa_yield: "100".to_string(),
    }
}

#[test]
fn test_query_dashboard() {
    let mut deps = mock_dependencies(&[Coin {
        denom: "uusd".to_string(),
        amount: Uint128::from(2000000u128),
    }]);
    let info = mock_info("addr0000", &[]);
    //setting up the required infoironment for the function call (inputs)

    let mock_config = get_mock_config(&deps);

    deps.querier.with_token_balances(&[(
        &"AT-uusd".to_string(),
        &[(&MOCK_CONTRACT_ADDR.to_string(), &Uint128::from(1000000u128))],
    )]);

    let msg = InstantiateMsg {
        owner_addr: String::from("owner"),
        stable_denom: "uusd".to_string(),
        capa_yield: "100".to_string(),
    };

    let info = mock_info(
        "addr0000",
        &[Coin {
            denom: "uusd".to_string(),
            amount: Uint128::from(INITIAL_DEPOSIT_AMOUNT),
        }],
    );

    // we can just call .unwrap() to assert this was a success
    let res = instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

    let msg = get_register_contracts(&deps, &mock_config);

    let info = mock_info("owner", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg.clone());
    match res {
        Ok(msg) => {
            assert_eq!(msg.attributes.len(), 0)
        }
        _ => panic!("DO NOT ENTER HERE"),
    }
    let info = mock_info(
        "addr0000",
        &[Coin {
            denom: "uusd".to_string(),
            amount: Uint128::from(55_555_555_000_000u128),
        }],
    );

    deps.querier.update_balance(
        MOCK_CONTRACT_ADDR,
        vec![Coin {
            denom: "uusd".to_string(),
            amount: Uint128::from(55_555_555_000_000u128),
        }],
    );

    let msg = QueryMsg::Dashboard {};

    let res = &query(deps.as_ref(), mock_env(), msg.clone());
    let bin: StdResult<DashboardResponse> = match res {
        Ok(r) => from_binary(r),
        Err(_e) => panic!("Error: {}", _e),
    };

    let total_value_locked: Uint256 = Uint256::zero();
    let cust_total_supply: Uint256 = Uint256::zero();
    let cust_nb_accounts: Uint256 = Uint256::from(2u128);
    let cust_avg_balance: Uint256 = Uint256::zero();
    let current_profit: Uint256 = Uint256::zero();
    let total_profit: Uint256 = Uint256::zero();

    match res {
        Ok(msg) => {
            assert_eq!(
                bin.unwrap(),
                DashboardResponse {
                    total_value_locked,
                    cust_total_supply,
                    cust_nb_accounts,
                    cust_avg_balance,
                    current_profit,
                    total_profit,
                }
            );
        }
        Err(msg) => println!("{}", msg),
        _ => panic!("DO NOT ENTER HERE"),
    }

    deps.querier.with_token_balances(&[
        (
            &"aterra_contract".to_string(),
            &[(
                &MOCK_CONTRACT_ADDR.to_string(),
                &Uint128::from(INITIAL_DEPOSIT_AMOUNT),
            )],
        ),
        (
            &"cterra_contract".to_string(),
            &[(
                &MOCK_CONTRACT_ADDR.to_string(),
                &Uint128::from(2 * INITIAL_DEPOSIT_AMOUNT / 3),
            )],
        ),
    ]);

    let msg = QueryMsg::Dashboard {};

    let res = &query(deps.as_ref(), mock_env(), msg.clone());
    let bin: StdResult<DashboardResponse> = match res {
        Ok(r) => from_binary(r),
        Err(_e) => panic!("Error: {}", _e),
    };

    let total_value_locked: Uint256 = Uint256::from(INITIAL_DEPOSIT_AMOUNT);
    let cust_total_supply: Uint256 = Uint256::from(2 * INITIAL_DEPOSIT_AMOUNT / 3);
    let cust_nb_accounts: Uint256 = Uint256::from(2u128);
    let cust_avg_balance: Uint256 = Uint256::from(33333333u128);
    let current_profit: Uint256 = Uint256::from(33333334u128);
    let total_profit: Uint256 = Uint256::zero();

    match res {
        Ok(msg) => {
            assert_eq!(
                bin.unwrap(),
                DashboardResponse {
                    total_value_locked,
                    cust_total_supply,
                    cust_nb_accounts,
                    cust_avg_balance,
                    current_profit,
                    total_profit,
                }
            );
        }
        Err(msg) => println!("{}", msg),
        _ => panic!("DO NOT ENTER HERE"),
    }
}

#[test]
fn test_query_config() {
    let mut deps = mock_dependencies(&[Coin {
        denom: "uusd".to_string(),
        amount: Uint128::from(2000000u128),
    }]);
    let info = mock_info("addr0000", &[]);
    //setting up the required infoironment for the function call (inputs)

    let mock_config = get_mock_config(&deps);

    deps.querier.with_token_balances(&[(
        &"AT-uusd".to_string(),
        &[(&MOCK_CONTRACT_ADDR.to_string(), &Uint128::from(1000000u128))],
    )]);

    let msg = InstantiateMsg {
        owner_addr: String::from("owner"),
        stable_denom: "uusd".to_string(),
        capa_yield: "100".to_string(),
    };

    let info = mock_info(
        "addr0000",
        &[Coin {
            denom: "uusd".to_string(),
            amount: Uint128::from(INITIAL_DEPOSIT_AMOUNT),
        }],
    );

    // we can just call .unwrap() to assert this was a success
    let res = instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

    let msg = get_register_contracts(&deps, &mock_config);

    let info = mock_info("owner", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg.clone());
    match res {
        Ok(msg) => {
            assert_eq!(msg.attributes.len(), 0)
        }
        _ => panic!("DO NOT ENTER HERE"),
    }

    let msg = QueryMsg::Config {};

    let res = &query(deps.as_ref(), mock_env(), msg.clone());
    let bin: StdResult<ConfigResponse> = match res {
        Ok(r) => from_binary(r),
        Err(_e) => panic!("Error: {}", _e),
    };

    let owner_addr = deps
        .api
        .addr_humanize(&mock_config.owner_addr)
        .unwrap()
        .to_string();

    let market_contract = deps
        .api
        .addr_humanize(&mock_config.market_contract)
        .unwrap()
        .to_string();

    let aterra_contract = deps
        .api
        .addr_humanize(&mock_config.aterra_contract)
        .unwrap()
        .to_string();

    let cterra_contract = deps
        .api
        .addr_humanize(&mock_config.cterra_contract)
        .unwrap()
        .to_string();

    let capacorp_contract = deps
        .api
        .addr_humanize(&mock_config.capacorp_contract)
        .unwrap()
        .to_string();

    let capa_contract = deps
        .api
        .addr_humanize(&mock_config.capa_contract)
        .unwrap()
        .to_string();

    let insurance_contract = deps
        .api
        .addr_humanize(&mock_config.insurance_contract)
        .unwrap()
        .to_string();

    match res {
        Ok(msg) => {
            assert_eq!(
                bin.unwrap(),
                ConfigResponse {
                    owner_addr: owner_addr,
                    market_contract: market_contract,
                    aterra_contract: aterra_contract,
                    cterra_contract: cterra_contract,
                    capacorp_contract: capacorp_contract,
                    capa_contract: capa_contract,
                    insurance_contract: insurance_contract,
                    stable_denom: "uusd".to_string(),
                    capa_yield: "100".to_string(),
                }
            );
        }
        Err(msg) => println!("{}", msg),
        _ => panic!("DO NOT ENTER HERE"),
    }
}

#[test]
fn test_query_corpaccount() {
    let mut deps = mock_dependencies(&[Coin {
        denom: "uusd".to_string(),
        amount: Uint128::from(2000000u128),
    }]);
    let info = mock_info("addr0000", &[]);
    //setting up the required infoironment for the function call (inputs)

    let mock_config = get_mock_config(&deps);

    deps.querier.with_token_balances(&[(
        &"AT-uusd".to_string(),
        &[(&MOCK_CONTRACT_ADDR.to_string(), &Uint128::from(1000000u128))],
    )]);

    let msg = InstantiateMsg {
        owner_addr: String::from("owner"),
        stable_denom: "uusd".to_string(),
        capa_yield: "100".to_string(),
    };

    let info = mock_info(
        "addr0000",
        &[Coin {
            denom: "uusd".to_string(),
            amount: Uint128::from(INITIAL_DEPOSIT_AMOUNT),
        }],
    );

    // we can just call .unwrap() to assert this was a success
    let res = instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

    let msg = get_register_contracts(&deps, &mock_config);

    let info = mock_info("owner", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg.clone());
    match res {
        Ok(msg) => {
            assert_eq!(msg.attributes.len(), 0)
        }
        _ => panic!("DO NOT ENTER HERE"),
    }

    let msg = QueryMsg::CorpAccounts {};

    let res = &query(deps.as_ref(), mock_env(), msg.clone());
    let bin: StdResult<Vec<String>> = match res {
        Ok(r) => from_binary(r),
        Err(_e) => panic!("Error: {}", _e),
    };

    let mut corp_accounts = Vec::new();
    corp_accounts.push("daniel".to_string());
    corp_accounts.push("bruno".to_string());

    match res {
        Ok(msg) => {
            assert_eq!(bin.unwrap(), corp_accounts);
        }
        Err(msg) => println!("{}", msg),
        _ => panic!("DO NOT ENTER HERE"),
    }
}
#[test]
fn test_query_availableharvest() {
    let mut deps = mock_dependencies(&[Coin {
        denom: "uusd".to_string(),
        amount: Uint128::from(2000000u128),
    }]);
    let info = mock_info("addr0000", &[]);
    //setting up the required infoironment for the function call (inputs)

    let mock_config = get_mock_config(&deps);

    deps.querier.with_token_balances(&[(
        &"AT-uusd".to_string(),
        &[(&MOCK_CONTRACT_ADDR.to_string(), &Uint128::from(1000000u128))],
    )]);

    let msg = InstantiateMsg {
        owner_addr: String::from("owner"),
        stable_denom: "uusd".to_string(),
        capa_yield: "100".to_string(),
    };

    let info = mock_info(
        "addr0000",
        &[Coin {
            denom: "uusd".to_string(),
            amount: Uint128::from(INITIAL_DEPOSIT_AMOUNT),
        }],
    );

    // we can just call .unwrap() to assert this was a success
    let res = instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

    let msg = get_register_contracts(&deps, &mock_config);

    let info = mock_info("owner", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg.clone());
    match res {
        Ok(msg) => {
            assert_eq!(msg.attributes.len(), 0)
        }
        _ => panic!("DO NOT ENTER HERE"),
    }

    let msg = QueryMsg::AvailableHarvest {
        account_addr: "addr0000".to_string(),
    };

    let res = &query(deps.as_ref(), mock_env(), msg.clone());
    let bin: StdResult<Uint256> = match res {
        Ok(r) => from_binary(r),
        Err(_e) => panic!("Error: {}", _e),
    };

    match res {
        Ok(msg) => {
            assert_eq!(bin.unwrap(), Uint256::zero());
        }
        Err(msg) => println!("{}", msg),
        _ => panic!("DO NOT ENTER HERE"),
    }
}
#[test]
fn test_query_harvestedsum() {
    let mut deps = mock_dependencies(&[Coin {
        denom: "uusd".to_string(),
        amount: Uint128::from(2000000u128),
    }]);
    let info = mock_info("addr0000", &[]);
    //setting up the required infoironment for the function call (inputs)

    let mock_config = get_mock_config(&deps);

    deps.querier.with_token_balances(&[(
        &"AT-uusd".to_string(),
        &[(&MOCK_CONTRACT_ADDR.to_string(), &Uint128::from(1000000u128))],
    )]);

    let msg = InstantiateMsg {
        owner_addr: String::from("owner"),
        stable_denom: "uusd".to_string(),
        capa_yield: "100".to_string(),
    };

    let info = mock_info(
        "addr0000",
        &[Coin {
            denom: "uusd".to_string(),
            amount: Uint128::from(INITIAL_DEPOSIT_AMOUNT),
        }],
    );

    // we can just call .unwrap() to assert this was a success
    let res = instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

    let msg = get_register_contracts(&deps, &mock_config);

    let info = mock_info("owner", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg.clone());
    match res {
        Ok(msg) => {
            assert_eq!(msg.attributes.len(), 0)
        }
        _ => panic!("DO NOT ENTER HERE"),
    }

    let msg = QueryMsg::HarvestedSum {
        account_addr: "addr0000".to_string(),
    };

    let res = &query(deps.as_ref(), mock_env(), msg.clone());
    let bin: StdResult<Uint256> = match res {
        Ok(r) => from_binary(r),
        Err(_e) => panic!("Error: {}", _e),
    };

    match res {
        Ok(msg) => {
            assert_eq!(bin.unwrap(), Uint256::zero());
        }
        Err(msg) => println!("{}", msg),
        _ => panic!("DO NOT ENTER HERE"),
    }
}
#[test]
fn test_query_token() {
    let mut deps = mock_dependencies(&[Coin {
        denom: "uusd".to_string(),
        amount: Uint128::from(2000000u128),
    }]);
    let info = mock_info("addr0000", &[]);
    //setting up the required infoironment for the function call (inputs)

    let mock_config = get_mock_config(&deps);

    deps.querier.with_token_balances(&[(
        &"AT-uusd".to_string(),
        &[(&MOCK_CONTRACT_ADDR.to_string(), &Uint128::from(1000000u128))],
    )]);

    let msg = InstantiateMsg {
        owner_addr: String::from("owner"),
        stable_denom: "uusd".to_string(),
        capa_yield: "100".to_string(),
    };

    let info = mock_info(
        "addr0000",
        &[Coin {
            denom: "uusd".to_string(),
            amount: Uint128::from(INITIAL_DEPOSIT_AMOUNT),
        }],
    );

    // we can just call .unwrap() to assert this was a success
    let res = instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

    let msg = get_register_contracts(&deps, &mock_config);

    let info = mock_info("owner", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg.clone());
    match res {
        Ok(msg) => {
            assert_eq!(msg.attributes.len(), 0)
        }
        _ => panic!("DO NOT ENTER HERE"),
    }

    let msg = QueryMsg::QueryToken {
        contract_addr: "aterra_contract".to_string(),
        account_addr: "owner".to_string(),
    };

    let res = &query(deps.as_ref(), mock_env(), msg.clone());
    let bin: StdResult<Uint256> = match res {
        Ok(r) => from_binary(r),
        Err(_e) => panic!("Error: {}", _e),
    };

    match res {
        Ok(msg) => {
            assert_eq!(bin.unwrap(), Uint256::zero());
        }
        Err(msg) => println!("{}", msg),
        _ => panic!("DO NOT ENTER HERE"),
    }
}
#[test]
fn test_query_cust_supply() {
    let mut deps = mock_dependencies(&[Coin {
        denom: "uusd".to_string(),
        amount: Uint128::from(2000000u128),
    }]);
    let info = mock_info("addr0000", &[]);
    //setting up the required infoironment for the function call (inputs)

    let mock_config = get_mock_config(&deps);

    deps.querier.with_token_balances(&[(
        &"AT-uusd".to_string(),
        &[(&MOCK_CONTRACT_ADDR.to_string(), &Uint128::from(1000000u128))],
    )]);

    let msg = InstantiateMsg {
        owner_addr: String::from("owner"),
        stable_denom: "uusd".to_string(),
        capa_yield: "100".to_string(),
    };

    let info = mock_info(
        "addr0000",
        &[Coin {
            denom: "uusd".to_string(),
            amount: Uint128::from(INITIAL_DEPOSIT_AMOUNT),
        }],
    );

    // we can just call .unwrap() to assert this was a success
    let res = instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

    let msg = get_register_contracts(&deps, &mock_config);

    let info = mock_info("owner", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg.clone());
    match res {
        Ok(msg) => {
            assert_eq!(msg.attributes.len(), 0)
        }
        _ => panic!("DO NOT ENTER HERE"),
    }

    let msg = QueryMsg::QueryCustSupply {
        contract_addr: "cterra_contract".to_string(),
    };

    deps.querier.with_token_balances(&[(
        &"cterra_contract".to_string(),
        &[(&MOCK_CONTRACT_ADDR.to_string(), &Uint128::from(1000000u128))],
    )]);
    let res = &query(deps.as_ref(), mock_env(), msg.clone());
    let bin: StdResult<Uint256> = match res {
        Ok(r) => from_binary(r),
        Err(_e) => panic!("Error: {}", _e),
    };

    match res {
        Ok(msg) => {
            assert_eq!(bin.unwrap(), Uint256::from(1000000u128));
        }
        Err(msg) => println!("{}", msg),
        _ => panic!("DO NOT ENTER HERE"),
    }
}
