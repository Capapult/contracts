use crate::deposit::{deposit, redeem_stable};
use crate::msg::{
    ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg, RedeemStableHookMsg, TokenInfoResponse,
};
use crate::querier::{
    calculate_aterra_profit, query_capacorp_all_accounts, query_capapult_exchange_rate,
    query_dashboard, query_harvest_value, query_harvested_sum, query_token_balance,
};
use crate::state::{
    read_config, read_profit, store_config, store_profit, store_state, Config, State,
};
use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::{
    attr, entry_point, from_binary, to_binary, Addr, Attribute, Binary, CosmosMsg, Deps, DepsMut,
    Env, MessageInfo, QueryRequest, Response, StdError, StdResult, Uint128, WasmMsg, WasmQuery,
};
use cosmwasm_storage::to_length_prefixed;
use cw20::{Cw20ExecuteMsg, Cw20ReceiveMsg};

pub const _1M_: u128 = 1000000;
pub const INITIAL_DEPOSIT_AMOUNT: u128 = 100000000;

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
            contract_addr: env.contract.address.into(),
            owner_addr: msg.owner_addr,
            stable_denom: msg.stable_denom,
            market_contract: String::from(""),
            aterra_contract: String::from(""),
            cterra_contract: String::from(""),
            capacorp_contract: String::from(""),
            capa_contract: String::from(""),
            insurance_contract: String::from(""),
        },
    )?;

    store_state(deps.storage, &State {})?;

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
            if contract_addr != config.cterra_contract {
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
    if !config.aterra_contract.is_empty()
        || !config.market_contract.is_empty()
        || !config.cterra_contract.is_empty()
        || !config.capacorp_contract.is_empty()
        || !config.capa_contract.is_empty()
        || !config.insurance_contract.is_empty()
    {
        return Err(StdError::generic_err("Unauthorized"));
    }

    // permission check
    if info.sender != config.owner_addr {
        return Err(StdError::generic_err("Unauthorized"));
    }

    deps.api.addr_validate(&market_contract)?;
    deps.api.addr_validate(&aterra_contract)?;
    deps.api.addr_validate(&cterra_contract)?;
    deps.api.addr_validate(&capacorp_contract)?;
    deps.api.addr_validate(&capa_contract)?;
    deps.api.addr_validate(&insurance_contract)?;

    config.market_contract = String::from(market_contract);
    config.aterra_contract = String::from(aterra_contract);
    config.cterra_contract = String::from(cterra_contract);
    config.capacorp_contract = String::from(capacorp_contract);
    config.capa_contract = String::from(capa_contract);
    config.insurance_contract = String::from(insurance_contract);

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
    if info.sender != config.owner_addr {
        return Err(StdError::generic_err("Unauthorized"));
    }

    if let Some(owner_addr) = owner_addr {
        config.owner_addr = owner_addr.into();
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
    }
}

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config: Config = read_config(deps.storage)?;
    Ok(ConfigResponse {
        owner_addr: config.owner_addr,
        market_contract: config.market_contract,
        aterra_contract: config.aterra_contract,
        cterra_contract: config.cterra_contract,
        capacorp_contract: config.capacorp_contract,
        capa_contract: config.capa_contract,
        insurance_contract: config.insurance_contract,
        stable_denom: config.stable_denom,
    })
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
        contract_addr: config.aterra_contract.clone(),
        funds: vec![],
        msg: to_binary(&Cw20ExecuteMsg::Transfer {
            recipient: config.insurance_contract,
            amount: insurance_amount.into(),
        })?,
    }));

    let insurance_str: String = insurance_amount.into();
    logs.push(attr("insurance", insurance_str));

    let stake_holders = query_capacorp_all_accounts(deps.as_ref())?;

    for stake_holder in stake_holders {
        let percent = query_token_balance(
            deps.as_ref(),
            &deps.api.addr_validate(&config.capacorp_contract)?,
            &stake_holder,
        )?;
        let share = profit_amount * percent * Decimal256::from_ratio(1, 100000);
        messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: config.aterra_contract.clone(),
            funds: vec![],
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: stake_holder.clone().into(),
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
    if info.sender != config.owner_addr {
        return Err(StdError::generic_err("Unauthorized"));
    }

    let res = deps.querier.query(&QueryRequest::Wasm(WasmQuery::Raw {
        contract_addr: config.cterra_contract.clone(),
        key: Binary::from(to_length_prefixed(b"token_info")),
    }))?;

    let token_info: TokenInfoResponse = from_binary(&res)?;
    let cust_total_supply = Uint256::from(token_info.total_supply);

    let mut profit = calculate_aterra_profit(
        deps.as_ref(),
        &env.contract.address,
        &deps.api.addr_validate(&config.aterra_contract)?,
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
