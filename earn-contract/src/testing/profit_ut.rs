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
fn proper_calculate_fees() {}

#[test]
fn proper_calculate_profit() {}

#[test]
fn proper_transfer_capacorp() {}

#[test]
fn not_authorized_distribute() {
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

    deps.querier.with_token_balances(&[(
        &"AT-uusd".to_string(),
        &[(
            &MOCK_CONTRACT_ADDR.to_string(),
            &Uint128::from(INITIAL_DEPOSIT_AMOUNT),
        )],
    )]);
    deps.querier.update_balance(
        MOCK_CONTRACT_ADDR,
        vec![Coin {
            denom: "uusd".to_string(),
            amount: Uint128::from(INITIAL_DEPOSIT_AMOUNT + 55_555_555_000_000u128),
        }],
    );
    let msg = ExecuteMsg::Deposit {};
    let res = execute(deps.as_mut(), mock_env(), info.clone(), msg.clone());
    match res {
        Ok(msg) => {
            assert_eq!(
                msg.attributes,
                vec![
                    attr("action", "deposit_stable"),
                    attr("depositor", "addr0000"),
                    attr("mint_amount", "55555555000000"),
                    attr("deposit_amount", "55555555000000"),
                ]
            );
        }
        Err(msg) => panic!("DO NOT ENTER HERE"),
        _ => panic!("DO NOT ENTER HERE"),
    }
    let msg = ExecuteMsg::Distribute {};
    let res = execute(deps.as_mut(), mock_env(), info.clone(), msg.clone());
    match res {
        Ok(msg) => panic!("DO NOT ENTER HERE"),
        Err(msg) => assert_eq!("Generic error: Unauthorized", msg.to_string()),
        _ => panic!("DO NOT ENTER HERE"),
    }
}

#[test]
fn too_little_profit_distribute() {
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
        "owner",
        &[Coin {
            denom: "uusd".to_string(),
            amount: Uint128::from(55_555_555_000_000u128),
        }],
    );

    deps.querier.with_token_balances(&[(
        &"AT-uusd".to_string(),
        &[(
            &MOCK_CONTRACT_ADDR.to_string(),
            &Uint128::from(INITIAL_DEPOSIT_AMOUNT),
        )],
    )]);
    deps.querier.update_balance(
        MOCK_CONTRACT_ADDR,
        vec![Coin {
            denom: "uusd".to_string(),
            amount: Uint128::from(INITIAL_DEPOSIT_AMOUNT + 55_555_555_000_000u128),
        }],
    );
    let msg = ExecuteMsg::Deposit {};
    let res = execute(deps.as_mut(), mock_env(), info.clone(), msg.clone());
    match res {
        Ok(msg) => {
            assert_eq!(
                msg.attributes,
                vec![
                    attr("action", "deposit_stable"),
                    attr("depositor", "owner"),
                    attr("mint_amount", "55555555000000"),
                    attr("deposit_amount", "55555555000000"),
                ]
            );
        }
        Err(msg) => panic!("DO NOT ENTER HERE"),
        _ => panic!("DO NOT ENTER HERE"),
    }
    let msg = ExecuteMsg::Distribute {};
    let res = execute(deps.as_mut(), mock_env(), info.clone(), msg.clone());
    match res {
        Ok(msg) => panic!("Should be an error here"),
        Err(StdError::GenericErr { msg, .. }) => {
            assert_eq!(msg, "Too little profit to distribute: 0")
        }
        _ => panic!("DO NOT ENTER HERE"),
    }
}
#[test]
fn proper_distribute() {
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
        "owner",
        &[Coin {
            denom: "uusd".to_string(),
            amount: Uint128::from(55_555_555_000_000u128),
        }],
    );

    deps.querier.with_token_balances(&[(
        &"AT-uusd".to_string(),
        &[(
            &MOCK_CONTRACT_ADDR.to_string(),
            &Uint128::from(INITIAL_DEPOSIT_AMOUNT),
        )],
    )]);
    deps.querier.update_balance(
        MOCK_CONTRACT_ADDR,
        vec![Coin {
            denom: "uusd".to_string(),
            amount: Uint128::from(INITIAL_DEPOSIT_AMOUNT + 55_555_555_000_000u128),
        }],
    );

    let msg = ExecuteMsg::Deposit {};
    let res = execute(deps.as_mut(), mock_env(), info.clone(), msg.clone());
    match res {
        Ok(msg) => {
            assert_eq!(
                msg.attributes,
                vec![
                    attr("action", "deposit_stable"),
                    attr("depositor", "owner"),
                    attr("mint_amount", "55555555000000"),
                    attr("deposit_amount", "55555555000000"),
                ]
            );
        }
        Err(msg) => panic!("DO NOT ENTER HERE"),
        _ => panic!("DO NOT ENTER HERE"),
    }

    deps.querier.with_token_balances(&[
        (
            &"aterra_contract".to_string(),
            &[(
                &MOCK_CONTRACT_ADDR.to_string(),
                &Uint128::from(100_555_555_000_000u128),
            )],
        ),
        (
            &"cterra_contract".to_string(),
            &[(
                &MOCK_CONTRACT_ADDR.to_string(),
                &Uint128::from(10_555_555_000_000u128),
            )],
        ),
    ]);
    let msg = ExecuteMsg::Distribute {};
    let res = execute(deps.as_mut(), mock_env(), info.clone(), msg.clone());
    match res {
        Ok(msg) => {
            assert_eq!(
                msg.attributes,
                vec![
                    attr("action", "distribute"),
                    attr("insurance", "0"),
                    attr("daniel", "0"),
                    attr("bruno", "0"),
                ]
            );
        }
        Err(msg) => panic!("{}", msg.to_string()),
        _ => panic!("DO NOT ENTER HERE"),
    }
}
