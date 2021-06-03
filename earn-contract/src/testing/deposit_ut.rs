use crate::contract::{handle, init, INITIAL_DEPOSIT_AMOUNT};
use crate::deposit::{deposit, redeem_stable};
use crate::msg::{HandleMsg, InitMsg, RedeemStableHookMsg};
use crate::state::{Config, State};

use crate::testing::mock_querier::mock_dependencies;
use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::testing::{mock_env, MockApi, MockQuerier, MockStorage, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{
    log, to_binary, Api, Coin, CosmosMsg, Extern, HumanAddr, LogAttribute, StdError, Uint128,
    WasmMsg,
};
use cw20::{Cw20CoinHuman, Cw20HandleMsg, Cw20ReceiveMsg, MinterResponse};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use terra_cosmwasm::{TerraQuery, TerraQueryWrapper, TerraRoute};

#[test]
fn proper_deposit() {
    let mut deps = mock_dependencies(
        20,
        &[Coin {
            denom: "uusd".to_string(),
            amount: Uint128::from(2000000u128),
        }],
    );
    let env = mock_env("addr0000", &[]);
    //setting up the required environment for the function call (inputs)
    let mock_config = Config {
        contract_addr: deps
            .api
            .canonical_address(&HumanAddr::from(MOCK_CONTRACT_ADDR))
            .unwrap(),
        owner_addr: deps
            .api
            .canonical_address(&HumanAddr::from("owner"))
            .unwrap(),
        aterra_contract: deps
            .api
            .canonical_address(&HumanAddr::from("AT-uusd"))
            .unwrap(),
        market_contract: deps
            .api
            .canonical_address(&HumanAddr::from("market"))
            .unwrap(),
        cterra_contract: deps
            .api
            .canonical_address(&HumanAddr::from("cterra_contract"))
            .unwrap(),
        capacorp_contract: deps
            .api
            .canonical_address(&HumanAddr::from("capacorp_contract"))
            .unwrap(),
        capa_contract: deps
            .api
            .canonical_address(&HumanAddr::from("capa_contract"))
            .unwrap(),
        insurance_contract: deps
            .api
            .canonical_address(&HumanAddr::from("insurance_contract"))
            .unwrap(),
        stable_denom: "uusd".to_string(),
    };
    deps.querier.with_token_balances(&[(
        &HumanAddr::from("AT-uusd"),
        &[(
            &HumanAddr::from(MOCK_CONTRACT_ADDR),
            &Uint128::from(1000000u128),
        )],
    )]);

    let msg = InitMsg {
        owner_addr: HumanAddr::from("owner"),
        stable_denom: "uusd".to_string(),
    };

    let env = mock_env(
        "addr0000",
        &[Coin {
            denom: "uusd".to_string(),
            amount: Uint128(INITIAL_DEPOSIT_AMOUNT),
        }],
    );

    // we can just call .unwrap() to assert this was a success
    let res = init(&mut deps, env.clone(), msg).unwrap();

    let msg = HandleMsg::RegisterContracts {
        market_contract: HumanAddr::from("market_contract"),
        aterra_contract: HumanAddr::from("aterra_contract"),
        cterra_contract: HumanAddr::from("cterra_contract"),
        capacorp_contract: HumanAddr::from("capacorp_contract"),
        capa_contract: HumanAddr::from("capa_contract"),
        insurance_contract: HumanAddr::from("insurance_contract"),
    };

    let env = mock_env(HumanAddr::from("owner"), &[]);
    let res = handle(&mut deps, env, msg.clone());
    match res {
        Ok(msg) => {
            assert_eq!(msg.log, vec![])
        }
        Err(StdError::GenericErr { msg, .. }) => {
            println!("{}", msg);
            assert_eq!(msg, "Deposit amount must be greater than 0 uusd");
        }
        Err(_err) => {
            println!("{}", _err)
        }
        _ => panic!("DO NOT ENTER HERE"),
    }

    let msg = HandleMsg::Deposit {};
    let env = mock_env(
        "addr0000",
        &[Coin {
            denom: "uust".to_string(),
            amount: Uint128::from(0u128),
        }],
    );

    let res = handle(&mut deps, env, msg.clone());
    match res {
        Err(StdError::GenericErr { msg, .. }) => {
            assert_eq!(msg, "Deposit amount must be greater than 0 uusd")
        }
        _ => panic!("DO NOT ENTER HERE"),
    }

    let env = mock_env(
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
    let res = handle(&mut deps, env.clone(), msg.clone());
    match res {
        Ok(msg) => {
            assert_eq!(
                msg.log,
                vec![
                    log("action", "deposit_stable"),
                    log("depositor", "addr0000"),
                    log("mint_amount", "55555555000000"),
                    log("deposit_amount", "55555555000000"),
                ]
            );
            /*
            assert_eq!(
                msg.messages,
                vec![CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: HumanAddr::from("aterra_contract"),
                    send: vec![],
                    msg: to_binary(&Cw20HandleMsg::Mint {
                        recipient: HumanAddr::from("addr0000"),
                        amount: Uint128::from(55_555_555_000_000u128),
                    })
                    .unwrap(),
                }),
                ]
            );*/
        }
        Err(StdError::GenericErr { msg, .. }) => {
            assert_eq!(msg, "Deposit amount must be greater than 0 uusd")
        }
        Err(_err) => {
            println!("{}", _err)
        }
        _ => panic!("DO NOT ENTER HERE"),
    }

    #[test]
    fn proper_withdraw() {
        // TODO test withdraw

        let message_base64 = to_binary(&RedeemStableHookMsg::RedeemStable {});
        assert_eq!(message_base64, "eyJyZWRlZW1fc3RhYmxlIjoge319");
    }
}
