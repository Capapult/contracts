#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};

use crate::state::{read_config, store_config, Config};
use cosmwasm_std::{
      Binary, CanonicalAddr, Deps, DepsMut,
    Env, MessageInfo, Response, StdError, StdResult, Uint128, 
};

pub const _1M_: u128 = 1000000;
pub const INITIAL_DEPOSIT_AMOUNT: u128 = 100 * _1M_;

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
            earn11: CanonicalAddr::from(vec![]),
            earn20: CanonicalAddr::from(vec![]),
        },
    )?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    match msg {
        ExecuteMsg::RegisterContracts { earn55, earn100 } => {
            register_contracts(deps, info, &earn55, &earn100)
        },
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    Ok(Binary::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    Ok(Response::default())
}

pub fn register_contracts(
    deps: DepsMut,
    info: MessageInfo,
    earn55: &str,
    earn100: &str,
) -> StdResult<Response> {
    let mut config: Config = read_config(deps.storage)?;
    if config.earn11 != CanonicalAddr::from(vec![]) || config.earn20 != CanonicalAddr::from(vec![])
    {
        return Err(StdError::generic_err("Unauthorized"));
    }

    // permission check
    if deps.api.addr_canonicalize(info.sender.as_str())? != config.owner_addr {
        return Err(StdError::generic_err("Unauthorized"));
    }

    deps.api.addr_validate(&earn55)?;
    deps.api.addr_validate(&earn100)?;

    config.earn11 = deps.api.addr_canonicalize(earn55)?;
    config.earn20 = deps.api.addr_canonicalize(earn100)?;

    store_config(deps.storage, &config)?;

    Ok(Response::default())
}
