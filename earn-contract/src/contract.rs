use crate::deposit::{deposit, redeem_stable};
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, RedeemStableHookMsg};
use crate::querier::{
    calculate_aterra_profit, query_capacorp_all_accounts, query_capapult_exchange_rate,
    query_dashboard, query_harvest_value, query_harvested_sum, query_token_balance,
    query_token_supply, query_config
};

use crate::state::{
    read_config, read_profit, store_config, store_profit, Config, 
};
use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::{
    attr, entry_point, from_binary, to_binary, Addr, Attribute, Binary, CanonicalAddr, CosmosMsg,
    Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult, Uint128, WasmMsg,
};
use cw20::{Cw20ExecuteMsg, Cw20ReceiveMsg};

pub const _1M_: u128 = 1000000;
pub const INITIAL_DEPOSIT_AMOUNT: u128 = 100*_1M_;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    let initial_deposit = info
        .funds
        .iter()
        .find(|c| c.denom == msg.stable_denom)
        .map(|c| c.amount)
        .unwrap_or_else(Uint128::zero);

    if initial_deposit != INITIAL_DEPOSIT_AMOUNT.into() {
        return Err(StdError::generic_err(format!(
            "Must deposit initial funds {:?}{:?}",
            INITIAL_DEPOSIT_AMOUNT, msg.stable_denom
        )));
    }
    store_profit(deps.storage, &Uint256::zero())?;

    let result = deps.api.addr_validate(&msg.owner_addr);
    match result {
        Ok(_x) => {}
        Err(_x) => {
            return Err(StdError::generic_err(
                "Owner address does not pass validation",
            ))
        }
    }

    store_config(
        deps.storage,
        &Config {
            contract_addr: deps.api.addr_canonicalize(env.contract.address.as_str())?,
            owner_addr: deps.api.addr_canonicalize(&msg.owner_addr)?,
            stable_denom: msg.stable_denom,
            market_contract: CanonicalAddr::from(vec![]),
            aterra_contract: CanonicalAddr::from(vec![]),
            cterra_contract: CanonicalAddr::from(vec![]),
            capacorp_contract: CanonicalAddr::from(vec![]),
            capa_contract: CanonicalAddr::from(vec![]),
            insurance_contract: CanonicalAddr::from(vec![]),
        },
    )?;

    Ok(Response::default())
}
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    match msg {
        ExecuteMsg::RegisterContracts {
            market_contract,
            aterra_contract,
            cterra_contract,
            capacorp_contract,
            capa_contract,
            insurance_contract,
        } => register_contracts(
            deps,
            info,
            &market_contract,
            &aterra_contract,
            &cterra_contract,
            &capacorp_contract,
            &capa_contract,
            &insurance_contract,
        ),
        ExecuteMsg::UpdateConfig { owner_addr } => update_config(deps, info, owner_addr),
        ExecuteMsg::Distribute {} => distribute(deps, env, info),
        ExecuteMsg::Deposit {} => deposit(deps, info),
        ExecuteMsg::Receive(msg) => receive_cw20(deps, env, info, msg),
    }
}

pub fn receive_cw20(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    cw20_msg: Cw20ReceiveMsg,
) -> StdResult<Response> {
    let contract_addr = info.sender;
    let msg = cw20_msg.msg;

    match from_binary(&msg)? {
        RedeemStableHookMsg::RedeemStable {} => {
            // only asset contract can execute this message
            let config: Config = read_config(deps.storage)?;
            if deps.api.addr_canonicalize(contract_addr.as_str())? != config.cterra_contract {
                return Err(StdError::generic_err("Unauthorized"));
            }
            let sender = deps.api.addr_validate(&cw20_msg.sender)?;
            redeem_stable(deps, env, sender, cw20_msg.amount)
        }
    }
}

pub fn register_contracts(
    deps: DepsMut,
    info: MessageInfo,
    market_contract: &str,
    aterra_contract: &str,
    cterra_contract: &str,
    capacorp_contract: &str,
    capa_contract: &str,
    insurance_contract: &str,
) -> StdResult<Response> {
    let mut config: Config = read_config(deps.storage)?;
    if config.aterra_contract != CanonicalAddr::from(vec![])
        || config.market_contract != CanonicalAddr::from(vec![])
        || config.cterra_contract != CanonicalAddr::from(vec![])
        || config.capacorp_contract != CanonicalAddr::from(vec![])
        || config.capa_contract != CanonicalAddr::from(vec![])
        || config.insurance_contract != CanonicalAddr::from(vec![])
    {
        return Err(StdError::generic_err("Unauthorized"));
    }

    // permission check
    if deps.api.addr_canonicalize(info.sender.as_str())? != config.owner_addr {
        return Err(StdError::generic_err("Unauthorized"));
    }

    deps.api.addr_validate(&market_contract)?;
    deps.api.addr_validate(&aterra_contract)?;
    deps.api.addr_validate(&cterra_contract)?;
    deps.api.addr_validate(&capacorp_contract)?;
    deps.api.addr_validate(&capa_contract)?;
    deps.api.addr_validate(&insurance_contract)?;

    config.market_contract = deps.api.addr_canonicalize(market_contract)?;
    config.aterra_contract = deps.api.addr_canonicalize(aterra_contract)?;
    config.cterra_contract = deps.api.addr_canonicalize(cterra_contract)?;
    config.capacorp_contract = deps.api.addr_canonicalize(capacorp_contract)?;
    config.capa_contract = deps.api.addr_canonicalize(capa_contract)?;
    config.insurance_contract = deps.api.addr_canonicalize(insurance_contract)?;

    store_config(deps.storage, &config)?;

    Ok(Response::default())
}

pub fn update_config(
    deps: DepsMut,
    info: MessageInfo,
    owner_addr: Option<Addr>,
) -> StdResult<Response> {
    let mut config: Config = read_config(deps.storage)?;

    // permission check
    if deps.api.addr_canonicalize(info.sender.as_str())? != config.owner_addr {
        return Err(StdError::generic_err("Unauthorized"));
    }

    if let Some(owner_addr) = owner_addr {
        config.owner_addr = deps.api.addr_canonicalize(owner_addr.as_str())?;
    }

    store_config(deps.storage, &config)?;
    Ok(Response::new().add_attribute("action", "update_config"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::ExchangeRate {} => to_binary(&query_capapult_exchange_rate(deps)?),
        QueryMsg::Dashboard {} => to_binary(&query_dashboard(deps)?),
        QueryMsg::CorpAccounts {} => to_binary(&query_capacorp_all_accounts(deps)?),
        QueryMsg::AvailableHarvest { account_addr } => {
            to_binary(&query_harvest_value(deps, account_addr)?)
        }
        QueryMsg::HarvestedSum { account_addr } => {
            to_binary(&query_harvested_sum(deps, account_addr)?)
        }
        QueryMsg::QueryToken {
            contract_addr,
            account_addr,
        } => to_binary(&query_token_balance(
            deps,
            &deps.api.addr_validate(contract_addr.as_str())?,
            &deps.api.addr_validate(account_addr.as_str())?,
        )?),
        QueryMsg::QueryCustSupply { contract_addr } => to_binary(&query_token_supply(
            deps,
            deps.api.addr_validate(contract_addr.as_str())?,
        )?),
    }
}


fn transfer_capacorp(
    deps: DepsMut,
    config: Config,
    insurance_amount: Uint256,
    profit_amount: Uint256,
) -> StdResult<Response> {
    let mut messages: Vec<CosmosMsg> = Vec::new();
    let mut logs: Vec<Attribute> = vec![attr("action", "distribute")];

    messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: deps.api.addr_humanize(&config.aterra_contract)?.to_string(),
        funds: vec![],
        msg: to_binary(&Cw20ExecuteMsg::Transfer {
            recipient: deps
                .api
                .addr_humanize(&config.insurance_contract)?
                .to_string(),
            amount: insurance_amount.into(),
        })?,
    }));

    let insurance_str: String = insurance_amount.into();
    logs.push(attr("insurance", insurance_str));

    let stake_holders = query_capacorp_all_accounts(deps.as_ref())?;

    for stake_holder in stake_holders {
        let percent = query_token_balance(
            deps.as_ref(),
            &deps.api.addr_humanize(&config.capacorp_contract)?,
            &deps.api.addr_validate(&stake_holder)?,
        )?;
        let share = profit_amount * percent * Decimal256::from_ratio(1, 100000);
        messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: deps.api.addr_humanize(&config.aterra_contract)?.to_string(),
            funds: vec![],
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: stake_holder.clone(),
                amount: share.into(),
            })?,
        }));

        let share_str: String = share.into();
        logs.push(attr(stake_holder.clone().as_str(), share_str));
    }
    let total_profit = read_profit(deps.storage)?;
    let total_profit = total_profit + profit_amount;
    store_profit(deps.storage, &total_profit)?;
    Ok(Response::new().add_messages(messages).add_attributes(logs))
}

pub fn distribute(deps: DepsMut, env: Env, info: MessageInfo) -> StdResult<Response> {
    let config: Config = read_config(deps.storage)?;

    // permission check
    if deps.api.addr_canonicalize(info.sender.as_str())? != config.owner_addr {
        return Err(StdError::generic_err("Unauthorized"));
    }

    let cust_total_supply = query_token_supply(
        deps.as_ref(),
        deps.api.addr_humanize(&config.cterra_contract)?,
    )?;

    let mut profit = calculate_aterra_profit(
        deps.as_ref(),
        &env.contract.address,
        &deps.api.addr_humanize(&config.aterra_contract)?,
        cust_total_supply,
    )?;
    /*  let tax_amount = compute_tax(deps, &Coin {
            denom: config.stable_denom.clone(),
            amount: profit.into(),
        })?;

        profit = profit - tax_amount;
    */
    let insurance_share = Decimal256::from_ratio(3, 100);
    let insurance_amount = profit * insurance_share;

    profit = profit - insurance_amount;

    // TODO: once tested take profit when at least there is at least 100 USD of profit
    // DURING TEST, profit taking can occur with only 0.1 UST
    if profit < Uint256::from(/* INITIAL_DEPOSIT_AMOUNT * */ _1M_ / 10) {
        return Err(StdError::GenericErr {
            msg: format!("Too little profit to distribute: {}", profit),
        });
    }

    let response = transfer_capacorp(deps, config, insurance_amount, profit)?;
    Ok(response)
}
