use crate::msg::{DepositStableHandleMsg, RedeemStableHookMsg};
use crate::querier::{
    query_harvest_value, compute_tax, deduct_tax, query_capapult_exchange_rate, query_exchange_rate, query_token_balance,
};
use crate::state::{
    read_config, read_total_deposit, store_total_claim, store_total_deposit, Config,
};
use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::{
    attr, to_binary, Addr, BankMsg, CanonicalAddr, Coin, CosmosMsg, DepsMut, Env, MessageInfo,
    Response, StdError, StdResult, Uint128, WasmMsg,
};
use cw20::Cw20ExecuteMsg;

extern crate base64;

pub fn deposit(deps: DepsMut, info: MessageInfo) -> StdResult<Response> {
    let config: Config = read_config(deps.storage)?;

    // Check base denom deposit
    let mut deposit_amount: Uint256 = info
        .funds
        .iter()
        .find(|c| c.denom == config.stable_denom)
        .map(|c| Uint256::from(c.amount))
        .unwrap_or_else(Uint256::zero);

    let deposit_coin = deduct_tax(
        deps.as_ref(),
        Coin {
            denom: config.stable_denom.clone(),
            amount: deposit_amount.into(),
        },
    )?;
    deposit_amount = Uint256::from(deposit_coin.amount);


    // Cannot deposit smallish amount
    if deposit_amount <= Uint256::from(1_000_000u128) {
        return Err(StdError::generic_err(format!(
            "Deposit amount must be greater than 1 UST {}",
            config.stable_denom,
        )));
    }

    let capa_exchange_rate: Decimal256 = query_capapult_exchange_rate(deps.as_ref())?;
    let mint_amount = deposit_amount / capa_exchange_rate;

    let sender: CanonicalAddr = deps.api.addr_canonicalize(info.sender.as_str())?;
    let mut current_deposit = read_total_deposit(deps.storage, &sender);
    if current_deposit == Uint256::from(0u128) {
        store_total_claim(deps.storage, &sender, &Uint256::from(0u128))?;
    }

    current_deposit += deposit_amount;
    store_total_deposit(deps.storage, &sender, &current_deposit)?;

    Ok(Response::new()
        .add_messages(vec![
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: deps.api.addr_humanize(&config.market_contract)?.to_string(),
                msg: to_binary(&DepositStableHandleMsg::DepositStable {})?,
                funds: vec![Coin {
                    denom: config.stable_denom.clone(),
                    amount: deposit_amount.into(),
                }],
            }),
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: deps.api.addr_humanize(&config.cterra_contract)?.to_string(),
                funds: vec![],
                msg: to_binary(&Cw20ExecuteMsg::Mint {
                    recipient: info.sender.to_string(),
                    amount: mint_amount.into(),
                })?,
            }),
        ])
        .add_attributes(vec![
            attr("action", "deposit_stable"),
            attr("depositor", info.sender),
            attr("mint_amount", mint_amount),
            attr("deposit_amount", deposit_amount),
        ]))
}

pub fn redeem_stable(
    deps: DepsMut,
    env: Env,
    sender: Addr,
    burn_amount: Uint128,
) -> StdResult<Response> {
    let config: Config = read_config(deps.storage)?;
    // Load anchor token exchange rate with updated state
    let capa_exchange_rate: Decimal256 = query_capapult_exchange_rate(deps.as_ref())?;
    let exchange_rate: Decimal256 = query_exchange_rate(deps.as_ref())?;

    let mut withdraw_amount = Uint256::from(burn_amount) * capa_exchange_rate;

    let tax_amount = compute_tax(
        deps.as_ref(),
        &Coin {
            denom: config.stable_denom.clone(),
            amount: withdraw_amount.into(),
        },
    )?;
    withdraw_amount = withdraw_amount - tax_amount;
    let tax_amount = compute_tax(
        deps.as_ref(),
        &Coin {
            denom: config.stable_denom.clone(),
            amount: withdraw_amount.into(),
        },
    )?;
    withdraw_amount = withdraw_amount - tax_amount;

    if withdraw_amount <= Uint256::from(1_000_000u128) {
        return Err(StdError::generic_err(format!(
            "Withdrawal amount must be greater than 1 UST {}",
            config.stable_denom,
        )));
    }

    let aust_burn_amount = withdraw_amount / exchange_rate;
    let aust_contract_address = deps.api.addr_humanize(&config.aterra_contract)?;

   let current_balance = query_token_balance(
        deps.as_ref(),
        &aust_contract_address,
        &env.contract.address,
    )?;

     if aust_burn_amount > current_balance {
         return Err(StdError::generic_err(format!(
             "Not enough aust available; redeem amount {} larger than current balance {}",
             aust_burn_amount, current_balance
         )));
     }

    let sender_canon: CanonicalAddr = deps.api.addr_canonicalize(sender.as_str())?;
    let mut current_deposit = read_total_deposit(deps.storage, &sender_canon);
    let user_claim: Uint256;

    let harvest_value = query_harvest_value(deps.as_ref(), sender.to_string())?;

    let burn_amount_ust = Uint256::from(burn_amount) * capa_exchange_rate;
    if harvest_value > burn_amount_ust   {
        user_claim = harvest_value - burn_amount_ust;
    } else {
        user_claim = harvest_value;
    }

    store_total_claim(deps.storage, &sender_canon, &user_claim)?;

    if current_deposit > burn_amount_ust {
        current_deposit = current_deposit - burn_amount_ust;
    } else {
        current_deposit = Uint256::from(0u128);
    }
    store_total_deposit(deps.storage, &sender_canon, &current_deposit)?;

    Ok(Response::new()
        .add_messages(vec![
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: deps.api.addr_humanize(&config.aterra_contract)?.to_string(),
                funds: vec![],
                msg: to_binary(&Cw20ExecuteMsg::Send {
                    contract: deps.api.addr_humanize(&config.market_contract)?.to_string(),
                    amount: aust_burn_amount.into(),
                    msg: to_binary(&RedeemStableHookMsg::RedeemStable {})?,
                })?,
            }),
            CosmosMsg::Bank(BankMsg::Send {
                to_address: sender.into(),
                amount: vec![Coin {
                    denom: config.stable_denom.clone(),
                    amount: withdraw_amount.into(),
                }],
            }),
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: deps.api.addr_humanize(&config.cterra_contract)?.to_string(),
                funds: vec![],
                msg: to_binary(&Cw20ExecuteMsg::Burn {
                    amount: burn_amount,
                })?,
            }),
        ])
        .add_attributes(vec![
            attr("action", "redeem_stable"),
            attr("burn_amount cust", burn_amount),
            attr("aust_burn_amount aust", aust_burn_amount),
            attr("withdraw_amount ust", withdraw_amount),
        ]))
}
