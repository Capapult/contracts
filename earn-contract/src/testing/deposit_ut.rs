use crate::contract::{execute, instantiate, INITIAL_DEPOSIT_AMOUNT};
use crate::deposit::redeem_stable;
use crate::msg::{ExecuteMsg, InstantiateMsg, RedeemStableHookMsg};
use crate::querier::query_token_balance;
use crate::state::Config;
use crate::testing::mock_querier::{mock_dependencies, WasmMockQuerier};
use cosmwasm_std::testing::{
    mock_env, mock_info, MockApi, MockQuerier, MockStorage, MOCK_CONTRACT_ADDR,
};
use cosmwasm_std::{
    attr, from_binary, to_binary, Api, Coin, Deps, DepsMut, MemoryStorage, StdError, StdResult,
    Uint128,
};

#[test]
fn too_small_deposit() {
    let mut deps = mock_dependencies(&[Coin {
        denom: "uusd".to_string(),
        amount: Uint128::from(2000000u128),
    }]);
    let info = mock_info("addr0000", &[]);
    //setting up the required infoironment for the function call (inputs)
    let mock_config = Config {
        contract_addr: String::from(MOCK_CONTRACT_ADDR),
        owner_addr: String::from("owner"),
        aterra_contract: String::from("AT-uusd"),
        market_contract: String::from("market"),
        cterra_contract: String::from("cterra_contract"),
        capacorp_contract: String::from("capacorp_contract"),
        capa_contract: String::from("capa_contract"),
        insurance_contract: String::from("insurance_contract"),
        stable_denom: "uusd".to_string(),
    };

    deps.querier.with_token_balances(&[(
        &"AT-uusd".to_string(),
        &[(&MOCK_CONTRACT_ADDR.to_string(), &Uint128::from(1000000u128))],
    )]);

    let msg = InstantiateMsg {
        owner_addr: String::from("owner"),
        stable_denom: "uusd".to_string(),
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

    let msg = ExecuteMsg::RegisterContracts {
        market_contract: String::from("market_contract"),
        aterra_contract: String::from("aterra_contract"),
        cterra_contract: String::from("cterra_contract"),
        capacorp_contract: String::from("capacorp_contract"),
        capa_contract: String::from("capa_contract"),
        insurance_contract: String::from("insurance_contract"),
    };

    let info = mock_info("owner", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg.clone());
    match res {
        Ok(msg) => {
            assert_eq!(msg.attributes.len(), 0)
        }
        _ => panic!("DO NOT ENTER HERE"),
    }

    let msg = ExecuteMsg::Deposit {};
    let info = mock_info(
        "addr0000",
        &[Coin {
            denom: "uust".to_string(),
            amount: Uint128::from(0u128),
        }],
    );

    let res = execute(deps.as_mut(), mock_env(), info.clone(), msg.clone());
    match res {
        Err(StdError::GenericErr { msg, .. }) => {
            assert_eq!(msg, "Deposit amount must be greater than 0 after tax uusd")
        }
        _ => panic!("DO NOT ENTER HERE"),
    }
}

#[test]
fn proper_deposit() {
    let mut deps = mock_dependencies(&[Coin {
        denom: "uusd".to_string(),
        amount: Uint128::from(2000000u128),
    }]);
    let info = mock_info("addr0000", &[]);
    //setting up the required infoironment for the function call (inputs)
    let mock_config = Config {
        contract_addr: String::from(MOCK_CONTRACT_ADDR),
        owner_addr: String::from("owner"),
        aterra_contract: String::from("AT-uusd"),
        market_contract: String::from("market"),
        cterra_contract: String::from("cterra_contract"),
        capacorp_contract: String::from("capacorp_contract"),
        capa_contract: String::from("capa_contract"),
        insurance_contract: String::from("insurance_contract"),
        stable_denom: "uusd".to_string(),
    };

    deps.querier.with_token_balances(&[(
        &"AT-uusd".to_string(),
        &[(&MOCK_CONTRACT_ADDR.to_string(), &Uint128::from(1000000u128))],
    )]);

    let msg = InstantiateMsg {
        owner_addr: String::from("owner"),
        stable_denom: "uusd".to_string(),
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

    let msg = ExecuteMsg::RegisterContracts {
        market_contract: String::from("market_contract"),
        aterra_contract: String::from("aterra_contract"),
        cterra_contract: String::from("cterra_contract"),
        capacorp_contract: String::from("capacorp_contract"),
        capa_contract: String::from("capa_contract"),
        insurance_contract: String::from("insurance_contract"),
    };

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
        Err(msg) => println!("{}", msg),
        _ => panic!("DO NOT ENTER HERE"),
    }
}

#[test]
fn withdraw_too_much() {
    let mut deps = mock_dependencies(&[Coin {
        denom: "uusd".to_string(),
        amount: Uint128::from(2000000u128),
    }]);
    let info = mock_info("addr0000", &[]);
    //setting up the required infoironment for the function call (inputs)
    let mock_config = Config {
        contract_addr: String::from(MOCK_CONTRACT_ADDR),
        owner_addr: String::from("owner"),
        aterra_contract: String::from("AT-uusd"),
        market_contract: String::from("market"),
        cterra_contract: String::from("cterra_contract"),
        capacorp_contract: String::from("capacorp_contract"),
        capa_contract: String::from("capa_contract"),
        insurance_contract: String::from("insurance_contract"),
        stable_denom: "uusd".to_string(),
    };

    deps.querier.with_token_balances(&[(
        &"AT-uusd".to_string(),
        &[(&MOCK_CONTRACT_ADDR.to_string(), &Uint128::from(1000000u128))],
    )]);

    let msg = InstantiateMsg {
        owner_addr: String::from("owner"),
        stable_denom: "uusd".to_string(),
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

    let msg = ExecuteMsg::RegisterContracts {
        market_contract: String::from("market_contract"),
        aterra_contract: String::from("aterra_contract"),
        cterra_contract: String::from("cterra_contract"),
        capacorp_contract: String::from("capacorp_contract"),
        capa_contract: String::from("capa_contract"),
        insurance_contract: String::from("insurance_contract"),
    };

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
        &"aterra_contract".to_string(),
        &[(
            &MOCK_CONTRACT_ADDR.to_string(),
            &Uint128::from(INITIAL_DEPOSIT_AMOUNT),
        )],
    )]);
    deps.querier.update_balance(
        MOCK_CONTRACT_ADDR,
        vec![Coin {
            denom: "uusd".to_string(),
            amount: Uint128::from(55_555_555_000_000u128),
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
        Err(msg) => println!("{}", msg),
        _ => panic!("DO NOT ENTER HERE"),
    }

    deps.querier.with_token_balances(&[(
        &"aterra_contract".to_string(),
        &[(
            &MOCK_CONTRACT_ADDR.to_string(),
            &Uint128::from(55555554750000u128),
        )],
    )]);

    let sender = deps.api.addr_validate(&"addr0000").unwrap();
    let res = redeem_stable(
        deps.as_mut(),
        mock_env(),
        sender,
        Uint128::from(55_555_555_000_000u128),
    );
    match res {
        Ok(_msg) => panic!("DO NOT ENTER HERE"),
        Err(msg) =>  assert_eq!(
            "Generic error: Not enough uusd available; redeem amount 55555555000000 larger than current balance 55555554750000", 
            msg.to_string()),
        _ => panic!("DO NOT ENTER HERE"),
    }
}

#[test]
fn withdraw_too_little() {
    let mut deps = mock_dependencies(&[Coin {
        denom: "uusd".to_string(),
        amount: Uint128::from(2000000u128),
    }]);
    let info = mock_info("addr0000", &[]);
    //setting up the required infoironment for the function call (inputs)
    let mock_config = Config {
        contract_addr: String::from(MOCK_CONTRACT_ADDR),
        owner_addr: String::from("owner"),
        aterra_contract: String::from("AT-uusd"),
        market_contract: String::from("market"),
        cterra_contract: String::from("cterra_contract"),
        capacorp_contract: String::from("capacorp_contract"),
        capa_contract: String::from("capa_contract"),
        insurance_contract: String::from("insurance_contract"),
        stable_denom: "uusd".to_string(),
    };

    deps.querier.with_token_balances(&[(
        &"aterra_contract".to_string(),
        &[(&MOCK_CONTRACT_ADDR.to_string(), &Uint128::from(1000000u128))],
    )]);

    let msg = InstantiateMsg {
        owner_addr: String::from("owner"),
        stable_denom: "uusd".to_string(),
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

    let msg = ExecuteMsg::RegisterContracts {
        market_contract: String::from("market_contract"),
        aterra_contract: String::from("aterra_contract"),
        cterra_contract: String::from("cterra_contract"),
        capacorp_contract: String::from("capacorp_contract"),
        capa_contract: String::from("capa_contract"),
        insurance_contract: String::from("insurance_contract"),
    };

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
            amount: Uint128::from(55_555_555_000_000u128),
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
        Err(msg) => println!("{}", msg),
        _ => panic!("DO NOT ENTER HERE"),
    }

    deps.querier.with_token_balances(&[(
        &"AT-uusd".to_string(),
        &[(
            &MOCK_CONTRACT_ADDR.to_string(),
            &Uint128::from(55555554750000u128),
        )],
    )]);

    let sender = deps.api.addr_validate(&"addr0000").unwrap();
    let res = redeem_stable(deps.as_mut(), mock_env(), sender, Uint128::zero());
    match res {
        Ok(msg) => panic!("DO NOT ENTER HERE"),
        Err(msg) => assert_eq!(
            "Generic error: Withdrawal amount must be greater than 0 after tax uusd",
            msg.to_string()
        ),
        _ => panic!("DO NOT ENTER HERE"),
    }
}

#[test]
fn proper_withdraw() {
    let mut deps = mock_dependencies(&[Coin {
        denom: "uusd".to_string(),
        amount: Uint128::from(2000000u128),
    }]);
    let info = mock_info("addr0000", &[]);
    //setting up the required infoironment for the function call (inputs)
    let mock_config = Config {
        contract_addr: String::from(MOCK_CONTRACT_ADDR),
        owner_addr: String::from("owner"),
        aterra_contract: String::from("AT-uusd"),
        market_contract: String::from("market"),
        cterra_contract: String::from("cterra_contract"),
        capacorp_contract: String::from("capacorp_contract"),
        capa_contract: String::from("capa_contract"),
        insurance_contract: String::from("insurance_contract"),
        stable_denom: "uusd".to_string(),
    };

    deps.querier.with_token_balances(&[(
        &"AT-uusd".to_string(),
        &[(&MOCK_CONTRACT_ADDR.to_string(), &Uint128::from(1000000u128))],
    )]);

    let msg = InstantiateMsg {
        owner_addr: String::from("owner"),
        stable_denom: "uusd".to_string(),
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

    let msg = ExecuteMsg::RegisterContracts {
        market_contract: String::from("market_contract"),
        aterra_contract: String::from("aterra_contract"),
        cterra_contract: String::from("cterra_contract"),
        capacorp_contract: String::from("capacorp_contract"),
        capa_contract: String::from("capa_contract"),
        insurance_contract: String::from("insurance_contract"),
    };

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
        &"aterra_contract".to_string(),
        &[(
            &MOCK_CONTRACT_ADDR.to_string(),
            &Uint128::from(INITIAL_DEPOSIT_AMOUNT),
        )],
    )]);
    deps.querier.update_balance(
        MOCK_CONTRACT_ADDR,
        vec![Coin {
            denom: "uusd".to_string(),
            amount: Uint128::from(55_555_555_000_000u128),
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
        Err(msg) => println!("{}", msg),
        _ => panic!("DO NOT ENTER HERE"),
    }

    deps.querier.with_token_balances(&[(
        &"aterra_contract".to_string(),
        &[(
            &MOCK_CONTRACT_ADDR.to_string(),
            &Uint128::from(55555554750000u128),
        )],
    )]);

    let sender = deps.api.addr_validate(&"addr0000").unwrap();
    let res = redeem_stable(
        deps.as_mut(),
        mock_env(),
        sender,
        Uint128::from(55555554750000u128),
    );
    match res {
        Ok(msg) => {
            assert_eq!(
                msg.attributes,
                vec![
                    attr("action", "redeem_stable"),
                    attr("burn_amount cust", "55555554750000"),
                    attr("aust_burn_amount aust", "55555554750000"),
                    attr("withdraw_amount ust", "55555554750000"),
                ]
            );
        }
        Err(msg) => panic!("Error: {}", msg),
        _ => panic!("DO NOT ENTER HERE"),
    }
}
