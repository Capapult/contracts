use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::{
    log, to_binary, Api, BankMsg, Coin, CosmosMsg, Env, Extern, HandleResponse, HandleResult,
    HumanAddr, Querier, StdError, StdResult, Storage, Uint128, WasmMsg,
};

pub fn deposit<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
) -> HandleResult {
    let config: Config = read_config(&deps.storage)?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![log("action", "deposit")],
        data: None,
    })
}

pub fn withdraw<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
) -> HandleResult {
    let config: Config = read_config(&deps.storage)?;
    
    Ok(HandleResponse {
        messages: vec![],
        log: vec![log("action", "withdraw")],
        data: None,
    })
}