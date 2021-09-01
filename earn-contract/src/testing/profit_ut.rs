use crate::contract::{execute, query, instantiate, INITIAL_DEPOSIT_AMOUNT};
use crate::msg::{ExecuteMsg, QueryMsg, InstantiateMsg, DashboardResponse};
use crate::state::Config;
use crate::testing::mock_querier::{mock_dependencies, WasmMockQuerier};
use cosmwasm_std::testing::{
    mock_env, mock_info, MockApi, MockQuerier, MockStorage, MOCK_CONTRACT_ADDR,
};
use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::{
    attr, from_binary, to_binary, Api, Coin, Deps, DepsMut, MemoryStorage, StdError, StdResult,
    Uint128,
};
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

#[test]
fn test_query_dashboard() {
    /*
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
    let msg = QueryMsg::Dashboard {};

    let res = &query(deps.as_ref(), mock_env(), msg.clone());    
    let bin : StdResult<DashboardResponse> = match res {
        Ok(r) => from_binary(r),
        Err(_e) => panic!("Error: {}", _e),
    };


    let  total_value_locked: Uint256 = Uint256::zero();
    let cust_total_supply: Uint256 = Uint256::zero();
    let cust_nb_accounts: Uint256 = Uint256::zero();
    let cust_avg_balance: Decimal256 = Decimal256::zero();
    let current_profit: Uint256 = Uint256::zero();
    let total_profit: Uint256 = Uint256::zero();

    match res {
        Ok(msg) => {
            assert_eq!(
                bin.unwrap(),
                DashboardResponse{
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
*/
}