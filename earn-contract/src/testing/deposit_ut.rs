use crate::contract::{execute, instantiate, INITIAL_DEPOSIT_AMOUNT};
use crate::deposit::{redeem_stable};
use crate::querier::{query_token_balance};
use crate::msg::{ExecuteMsg, InstantiateMsg, RedeemStableHookMsg};
use crate::state::Config;
use crate::testing::mock_querier::{mock_dependencies,WasmMockQuerier};
use cosmwasm_std::testing::{mock_info, mock_env, MockApi, MockQuerier, MockStorage, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{Deps, DepsMut, from_binary, attr, to_binary, Api, Coin, StdResult, StdError, Uint128, MemoryStorage};

fn config(deps: DepsMut) -> Config {
    Config {
        contract_addr: String::from(MOCK_CONTRACT_ADDR),
        owner_addr: String::from("owner"),
        aterra_contract: String::from("AT-uusd"),
        market_contract: String::from("market"),
        cterra_contract: String::from("cterra_contract"),
        capacorp_contract: String::from("capacorp_contract"),
        capa_contract: String::from("capa_contract"),
        insurance_contract: String::from("insurance_contract"),
        stable_denom: "uusd".to_string(),
    }
}


fn init_test() -> DepsMut {
    let mut deps = mock_dependencies(
        20,
        &[Coin {
            denom: "uusd".to_string(),
            amount: Uint128::from(2000000u128),
        }],
    );
    let info = mock_info("addr0000", &[]);
    //setting up the required infoironment for the function call (inputs)
    let mock_config = config(deps);

    deps.querier.with_token_balances(&[(
        &HumanAddr::from("AT-uusd"),
        &[(
            &HumanAddr::from(MOCK_CONTRACT_ADDR),
            &Uint128::from(1000000u128),
        )],
    )]);

    let msg = InstantiateMsg {
        owner_addr: String::from("owner"),
        stable_denom: "uusd".to_string(),
    };

    let info = mock_info(
        "addr0000",
        &[Coin {
            denom: "uusd".to_string(),
            amount: Uint128(INITIAL_DEPOSIT_AMOUNT),
        }],
    );

    // we can just call .unwrap() to assert this was a success
    let res = instantiate(deps, mock_env(), info.clone(), msg).unwrap();

    let msg = ExecuteMsg::RegisterContracts {
        market_contract: String::from("market_contract"),
        aterra_contract: String::from("aterra_contract"),
        cterra_contract: String::from("cterra_contract"),
        capacorp_contract: String::from("capacorp_contract"),
        capa_contract: String::from("capa_contract"),
        insurance_contract: String::from("insurance_contract"),
    };

    let info = mock_info(String::from("owner"), &[]);
    let res = execute(deps, mock_env(), info, msg.clone());
    match res {
        Ok(msg) => {
            assert_eq!(msg.attributes, vec![])
        }
        _ => panic!("DO NOT ENTER HERE"),
    }    
    deps
}
#[test]
fn too_small_deposit() {
    let mut deps = init_test();

    let msg = ExecuteMsg::Deposit {};
    let info = mock_info(
        "addr0000",
        &[Coin {
            denom: "uust".to_string(),
            amount: Uint128::from(0u128),
        }],
    );

    let res = execute(deps, mock_env(), info.clone(), msg.clone());
    match res {
        Err(StdError::GenericErr { msg, .. }) => {
            assert_eq!(msg, "Deposit amount must be greater than 0 after tax uusd")
        }
        _ => panic!("DO NOT ENTER HERE"),
    }    
}

#[test]
fn proper_deposit() {
    
    let mut deps = init_test();

    let info = mock_info(
        "addr0000",
        &[Coin {
            denom: "uusd".to_string(),
            amount: Uint128::from(55_555_555_000_000u128),
        }],
    );

    deps.querier.with_token_balances(&[(
        &HumanAddr::from("AT-uusd"),
        &[(
            &HumanAddr::from(MOCK_CONTRACT_ADDR),
            &Uint128::from(INITIAL_DEPOSIT_AMOUNT),
        )],
    )]);
    deps.querier.update_balance(
        HumanAddr::from(MOCK_CONTRACT_ADDR),
        vec![Coin {
            denom: "uusd".to_string(),
            amount: Uint128::from(INITIAL_DEPOSIT_AMOUNT + 55_555_555_000_000u128),
        }],
    );
    let msg = ExecuteMsg::Deposit {};
    let res = execute(&mut deps, mock_env(), info.clone(),  msg.clone());
    match res {
        Ok(msg) => {
            assert_eq!(
                msg.attributes,
                vec![
                    attr("action", "deposit_stable"),
                    attr("depositor", "addr0000"),
                    attr("mint_amount",    "55555555000000"),
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
    let mut deps = init_test();

    let info = mock_info(
        "addr0000",
        &[Coin {
            denom: "uusd".to_string(),
            amount: Uint128::from(55_555_555_000_000u128),
        }],
    );

    deps.querier.with_token_balances(&[(
        &HumanAddr::from("AT-uusd"),
        &[(
            &HumanAddr::from(MOCK_CONTRACT_ADDR),
            &Uint128::from(INITIAL_DEPOSIT_AMOUNT),
        )],
    )]);
    deps.querier.update_balance(
        HumanAddr::from(MOCK_CONTRACT_ADDR),
        vec![Coin {
            denom: "uusd".to_string(),
            amount: Uint128::from(55_555_555_000_000u128),
        }],
    );
    let msg = ExecuteMsg::Deposit {};
    let res = execute(&mut deps, mock_env(), info.clone(), msg.clone());
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
        },        
        Err(msg) => println!("{}", msg),
        _ => panic!("DO NOT ENTER HERE"),
    }

    deps.querier.with_token_balances(&[(
        &HumanAddr::from("aterra_contract"),
        &[(
            &info.contract.address,
            &Uint128::from(55555554750000u128),
        )],
    )]);
    
    let res = redeem_stable(deps.as_ref(), mock_env(), String::from("addr0000") , Uint128::from(55_555_555_000_000u128)); 
    match res {
        Ok(msg) => panic!("DO NOT ENTER HERE"),
        Err(msg) =>  assert_eq!(
            "Generic error: Not enough uusd available; redeem amount 55555555000000 larger than current balance 55555554750000", 
            msg.to_string()),
        _ => panic!("DO NOT ENTER HERE"),
    }
}

#[test]
fn withdraw_too_little() {
    let mut deps = init_test();

    let info = mock_info(
        "addr0000",
        &[Coin {
            denom: "uusd".to_string(),
            amount: Uint128::from(55_555_555_000_000u128),
        }],
    );

    deps.querier.with_token_balances(&[(
        &HumanAddr::from("AT-uusd"),
        &[(
            &HumanAddr::from(MOCK_CONTRACT_ADDR),
            &Uint128::from(INITIAL_DEPOSIT_AMOUNT),
        )],
    )]);
    deps.querier.update_balance(
        HumanAddr::from(MOCK_CONTRACT_ADDR),
        vec![Coin {
            denom: "uusd".to_string(),
            amount: Uint128::from(55_555_555_000_000u128),
        }],
    );
    let msg = ExecuteMsg::Deposit {};
    let res = execute(&mut deps, mock_env(), info.clone(), msg.clone());
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
        },        
        Err(msg) => println!("{}", msg),
        _ => panic!("DO NOT ENTER HERE"),
    }

    deps.querier.with_token_balances(&[(
        &HumanAddr::from("aterra_contract"),
        &[(
            &info.contract.address,
            &Uint128::from(55555554750000u128),
        )],
    )]);

    let res = redeem_stable(deps, mock_env(), HumanAddr::from("addr0000") , Uint128::zero()); 
    match res {
        Ok(msg) => panic!("DO NOT ENTER HERE"),
        Err(msg) =>  assert_eq!(
            "Generic error: Withdrawal amount must be greater than 0 after tax uusd", 
            msg.to_string()),
        _ => panic!("DO NOT ENTER HERE"),
    }
}
#[test]
fn proper_withdraw() {
    let mut deps = init_test();

    let info = mock_info(
        "addr0000",
        &[Coin {
            denom: "uusd".to_string(),
            amount: Uint128::from(55_555_555_000_000u128),
        }],
    );

    deps.querier.with_token_balances(&[(
        &HumanAddr::from("AT-uusd"),
        &[(
            &HumanAddr::from(MOCK_CONTRACT_ADDR),
            &Uint128::from(INITIAL_DEPOSIT_AMOUNT),
        )],
    )]);
    deps.querier.update_balance(
        HumanAddr::from(MOCK_CONTRACT_ADDR),
        vec![Coin {
            denom: "uusd".to_string(),
            amount: Uint128::from(55_555_555_000_000u128),
        }],
    );
    let msg = ExecuteMsg::Deposit {};
    let res = execute(&mut deps, mock_env(), info.clone(), msg.clone());
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
        },        
        Err(msg) => println!("{}", msg),
        _ => panic!("DO NOT ENTER HERE"),
    }

    deps.querier.with_token_balances(&[(
        &HumanAddr::from("aterra_contract"),
        &[(
            &info.contract.address,
            &Uint128::from(55555554750000u128),
        )],
    )]);
    
    let res = redeem_stable(deps, mock_env(),  HumanAddr::from("addr0000") , Uint128::from(55555554750000u128)); 
    match res {
        Ok(msg) => {
            assert_eq!(
                msg.attributes,
                vec![
                    attr("action", "redeem_stable"),
                 attr("burn_amount cust", 55555554750000u128),
                 attr("aust_burn_amount aust", 55555554750000u128),
                 attr("withdraw_amount ust", 55555554750000u128),
                ]
            );
        },   
        _ => panic!("DO NOT ENTER HERE"),
    }
}