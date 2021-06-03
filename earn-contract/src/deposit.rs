use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::{
    log, from_binary, to_binary, Binary, Api, BankMsg, Coin, CosmosMsg, Env, Extern, HandleResponse, HandleResult,
    HumanAddr, Querier, StdError, StdResult, Storage, Uint128, WasmMsg,QueryRequest, WasmQuery
};
use cw20::Cw20HandleMsg;
use crate::state::{read_config, Config, State};
use cosmwasm_storage::to_length_prefixed;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt;
use crate::msg::{ DepositStableHandleMsg, RedeemStableHookMsg};

extern crate base64;

pub fn deposit<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
) -> HandleResult {
    let config: Config = read_config(&deps.storage)?;

  //  println!("before deposit_amount: ");
    // Check base denom deposit
    let deposit_amount: Uint256 = env
        .message
        .sent_funds
        .iter()
        .find(|c| c.denom == config.stable_denom)
        .map(|c| Uint256::from(c.amount))
        .unwrap_or_else(Uint256::zero);

  //      println!("deposit_amount: {}", deposit_amount);

    // Cannot deposit zero amount
    if deposit_amount.is_zero() {
        return Err(StdError::generic_err(format!(
            "Deposit amount must be greater than 0 {}",
            config.stable_denom,
        )));
    }
  //  println!("deposit_amount: after is_zero");

    // TODO: mint amount has to be calculated according to capapult exchange rate curve
    let mint_amount = deposit_amount.clone();

  //  println!("mint_amount: {}", mint_amount);
  //  println!("aterra_contract: {}", config.aterra_contract);
  //  println!("cterra_contract: {}", config.cterra_contract);

    Ok(HandleResponse {
        messages: vec![
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: deps.api.human_address(&config.market_contract)?,
                msg: to_binary(&DepositStableHandleMsg::DepositStable {                    
                })?,
                send: vec![
                    Coin {
                        denom: config.stable_denom,
                        amount: deposit_amount.into(),
                    },
                ]
            }),
            CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: deps.api.human_address(&config.cterra_contract)?,
            send: vec![],
            msg: to_binary(&Cw20HandleMsg::Mint {
                recipient: env.message.sender.clone(),
                amount: mint_amount.into(),
            })?,
        })],
        log: vec![
            log("action", "deposit_stable"),
            log("depositor", env.message.sender),
            log("mint_amount", mint_amount),
            log("deposit_amount", deposit_amount),
        ],
        data: None,
    })
}


pub fn redeem_stable<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    sender: HumanAddr,
    burn_amount: Uint128,
) -> HandleResult {

    let config: Config = read_config(&deps.storage)?;
    

    // Load anchor token exchange rate with updated state
   // let exchange_rate  = query_exchange_rate(deps, &config, None)?;
    let redeem_amount = Uint256::from(burn_amount);// * exchange_rate;

    println!("redeem_amount: {}", redeem_amount);

    let current_balance = query_token_balance(
        &deps,
        &deps.api.human_address(&config.cterra_contract)?,
        &env.message.sender,
    )?;

    // Assert redeem amount
   // assert_redeem_amount(&config, current_balance, redeem_amount)?;

 Ok(HandleResponse {
        messages: vec![
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: deps.api.human_address(&config.aterra_contract)?,
                send: vec![],
                msg: to_binary(&Cw20HandleMsg::Send {
                        contract: deps.api.human_address(&config.market_contract)?,
                        amount: burn_amount.into(),
                        msg: Some(to_binary(&RedeemStableHookMsg::RedeemStable {})?),
                })?,
            }),
            CosmosMsg::Bank(BankMsg::Send {
                from_address: env.contract.address,
                to_address: sender.clone(),
                amount: vec![
                    Coin {
                        denom: config.stable_denom.clone(),
                        amount: burn_amount.into(),
                    }
                ],
            }),  
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: deps.api.human_address(&config.cterra_contract)?,
                send: vec![],
                msg: to_binary(&Cw20HandleMsg::Burn {
                    amount: burn_amount,
                })?,
            }),          
        ],
        log: vec![
            log("action", "redeem_stable"),
            log("burn_amount", burn_amount),
         //   log("redeem_amount", redeem_amount),
        ],
        data: None,
    })
}

fn assert_redeem_amount(
    config: &Config,
    current_balance: Uint256,
    redeem_amount: Uint256,
) -> StdResult<()> {
    let current_balance = Decimal256::from_uint256(current_balance);
    let redeem_amount = Decimal256::from_uint256(redeem_amount);
    if redeem_amount  > current_balance {
        return Err(StdError::generic_err(format!(
            "Not enough {} available; borrow demand too high",
            config.stable_denom
        )));
    }

    return Ok(());
}


pub fn query_token_balance<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    contract_addr: &HumanAddr,
    account_addr: &HumanAddr,
) -> StdResult<Uint256> {
    // load balance form the token contract
    let res: Binary = deps
        .querier
        .query(&QueryRequest::Wasm(WasmQuery::Raw {
            contract_addr: HumanAddr::from(contract_addr),
            key: Binary::from(concat(
                &to_length_prefixed(b"balance").to_vec(),
                (deps.api.canonical_address(&account_addr)?).as_slice(),
            )),
        }))
        .unwrap_or_else(|_| to_binary(&Uint128::zero()).unwrap());

    let balance: Uint128 = from_binary(&res)?;
    Ok(balance.into())
}

#[inline]
fn concat(namespace: &[u8], key: &[u8]) -> Vec<u8> {
    let mut k = namespace.to_vec();
    k.extend_from_slice(key);
    k
}
