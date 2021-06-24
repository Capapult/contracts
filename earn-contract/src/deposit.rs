use crate::msg::{DepositStableHandleMsg, RedeemStableHookMsg};
use crate::querier::{
    deduct_tax, compute_tax, query_capapult_exchange_rate, query_exchange_rate, query_token_balance,
};
use crate::state::{read_config, Config};
use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::{
    log, to_binary, Api, BankMsg, Coin, CosmosMsg, Env, Extern, HandleResponse, HandleResult,
    HumanAddr, Querier, StdError, StdResult, Storage, Uint128, WasmMsg,
};
use cw20::Cw20HandleMsg;

extern crate base64;

pub fn deposit<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
) -> HandleResult {
    let config: Config = read_config(&deps.storage)?;

    //  println!("before deposit_amount: ");
    // Check base denom deposit
    let mut deposit_amount: Uint256 = env
        .message
        .sent_funds
        .iter()
        .find(|c| c.denom == config.stable_denom)
        .map(|c| Uint256::from(c.amount))
        .unwrap_or_else(Uint256::zero);

    let deposit_coin = deduct_tax(
        deps,
        Coin {
            denom: config.stable_denom.clone(),
            amount: deposit_amount.into(),
        },
    )?;
    deposit_amount = Uint256::from(deposit_coin.amount);

    // Cannot deposit smallish amount
    if deposit_amount <= Uint256::from(0u128) {
        return Err(StdError::generic_err(format!(
            "Deposit amount must be greater than 0 after tax {}",
            config.stable_denom,
        )));
    }

    let capa_exchange_rate: Decimal256 = query_capapult_exchange_rate(deps)?;
    let mint_amount = deposit_amount / capa_exchange_rate;

    Ok(HandleResponse {
        messages: vec![
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: deps.api.human_address(&config.market_contract)?,
                msg: to_binary(&DepositStableHandleMsg::DepositStable {})?,
                send: vec![Coin {
                    denom: config.stable_denom.clone(),
                    amount: deposit_amount.into(),
                }],
            }),
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: deps.api.human_address(&config.cterra_contract)?,
                send: vec![],
                msg: to_binary(&Cw20HandleMsg::Mint {
                    recipient: env.message.sender.clone(),
                    amount: mint_amount.into(),
                })?,
            }),
        ],
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
    let capa_exchange_rate: Decimal256 = query_capapult_exchange_rate(deps)?;
    let exchange_rate: Decimal256 = query_exchange_rate(deps)?;

    let mut withdraw_amount = Uint256::from(burn_amount) * capa_exchange_rate;
    
    let redeem_amount = withdraw_amount / exchange_rate;

    let tax_amount = compute_tax(deps, &Coin {
        denom: config.stable_denom.clone(),
        amount: withdraw_amount.into(),
    })?;
    withdraw_amount = withdraw_amount - tax_amount - tax_amount;
    if withdraw_amount <= Uint256::from(00u128) {
        return Err(StdError::generic_err(format!(
            "Withdrawal amount must be greater than 0 after tax {}",
            config.stable_denom.clone(),
        )));
    }

    let current_balance = query_token_balance(
        &deps,
        &deps.api.human_address(&config.aterra_contract)?,
        &env.contract.address,
    )?;
    // Assert redeem amount
    assert_redeem_amount(&config, current_balance, redeem_amount)?;

    Ok(HandleResponse {
        messages: vec![
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: deps.api.human_address(&config.aterra_contract)?,
                send: vec![],
                msg: to_binary(&Cw20HandleMsg::Send {
                    contract: deps.api.human_address(&config.market_contract)?,
                    amount: redeem_amount.into(),
                    msg: Some(to_binary(&RedeemStableHookMsg::RedeemStable {})?),
                })?,
            }),
            CosmosMsg::Bank(BankMsg::Send {
                from_address: env.contract.address,
                to_address: sender,
                amount: vec![
                        Coin {
                            denom: config.stable_denom,
                            amount: withdraw_amount.into(),
                        },
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
            log("burn_amount cust", burn_amount),
            log("redeem_amount aust", redeem_amount),
            log("withdraw_amount ust", withdraw_amount),
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
    if redeem_amount > current_balance {
        return Err(StdError::generic_err(format!(
            "Not enough {} available; redeem amount {} larger than current balance {}",
            config.stable_denom, redeem_amount, current_balance
        )));
    }

    return Ok(());
}
