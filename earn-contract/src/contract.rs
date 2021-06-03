use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::{
    from_binary, log, to_binary, Api, BankMsg, Binary, CanonicalAddr, Coin, CosmosMsg, Env, Extern,
    HandleResponse, HandleResult, HumanAddr, InitResponse, InitResult, MigrateResponse,
    MigrateResult, Querier, StdError, StdResult, Storage, Uint128, WasmMsg,
};
use cw20::{Cw20ReceiveMsg};

use crate::msg::{ HandleMsg, InitMsg, QueryMsg, ConfigResponse, RedeemStableHookMsg};
use crate::state::{read_config, store_config, read_state, store_state, Config, State};
use crate::deposit::{deposit, redeem_stable};

pub const INITIAL_DEPOSIT_AMOUNT: u128 = 1000000;

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> InitResult {
    let initial_deposit = env
        .message
        .sent_funds
        .iter()
        .find(|c| c.denom == msg.stable_denom)
        .map(|c| c.amount)
        .unwrap_or_else(|| Uint128::zero());

    if initial_deposit != Uint128(INITIAL_DEPOSIT_AMOUNT) {
        return Err(StdError::generic_err(format!(
            "Must deposit initial funds {:?}{:?}",
            INITIAL_DEPOSIT_AMOUNT,
            msg.stable_denom.clone()
        )));
    }
    store_config(
        &mut deps.storage,
        &Config {
            contract_addr: deps.api.canonical_address(&env.contract.address)?,
            owner_addr: deps.api.canonical_address(&msg.owner_addr)?,
            stable_denom: msg.stable_denom.clone(),
            market_contract: CanonicalAddr::default(),
            aterra_contract: CanonicalAddr::default(),
            cterra_contract: CanonicalAddr::default(),
            capacorp_contract: CanonicalAddr::default(),
            capa_contract: CanonicalAddr::default(),
            insurance_contract: CanonicalAddr::default(),
        },
    )?;

    store_state(&mut deps.storage, &State {})?;

    Ok(InitResponse::default())
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    match msg {
        HandleMsg::RegisterContracts {
            market_contract,
            aterra_contract,
            cterra_contract,
            capacorp_contract,
            capa_contract,
            insurance_contract,
        } => register_contracts(
            deps, env,
            market_contract,
            aterra_contract,
            cterra_contract,
            capacorp_contract,
            capa_contract,
            insurance_contract,
        ),
        HandleMsg::UpdateConfig { owner_addr } => update_config(deps, env, owner_addr),
        HandleMsg::Distribute { owner_addr } => distribute(deps, env, owner_addr),
        HandleMsg::Deposit {} => deposit(deps, env),
        HandleMsg::Receive(msg) => receive_cw20(deps, env, msg),
    }
}

pub fn receive_cw20<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    cw20_msg: Cw20ReceiveMsg,
) -> HandleResult {
    let contract_addr = env.message.sender.clone();
    if let Some(msg) = cw20_msg.msg {
        match from_binary(&msg)? {
            RedeemStableHookMsg::RedeemStable {} => {
                // only asset contract can execute this message
                let config: Config = read_config(&deps.storage)?;
                if deps.api.canonical_address(&contract_addr)? != config.cterra_contract {
                    return Err(StdError::unauthorized());
                }

                redeem_stable(deps, env, cw20_msg.sender, cw20_msg.amount)
            }
        }
    } else {
        Err(StdError::generic_err(
            "Invalid request: \"redeem stable\" message not included in request",
        ))
    }
}

pub fn register_contracts<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    market_contract: HumanAddr,
    aterra_contract: HumanAddr,
    cterra_contract: HumanAddr,
    capacorp_contract: HumanAddr,
    capa_contract: HumanAddr,
    insurance_contract: HumanAddr,
) -> HandleResult {
    let mut config: Config = read_config(&deps.storage)?;
    if config.aterra_contract != CanonicalAddr::default()
    || config.market_contract != CanonicalAddr::default()
        || config.cterra_contract != CanonicalAddr::default()
        || config.capacorp_contract != CanonicalAddr::default()
        || config.capa_contract != CanonicalAddr::default()
        || config.insurance_contract != CanonicalAddr::default()
    {
        return Err(StdError::unauthorized());
    }

    // permission check
    if deps.api.canonical_address(&env.message.sender)? != config.owner_addr {
        return Err(StdError::unauthorized());
    }

    config.market_contract = deps.api.canonical_address(&market_contract)?;
    config.aterra_contract = deps.api.canonical_address(&aterra_contract)?;
    config.cterra_contract = deps.api.canonical_address(&cterra_contract)?;
    config.capacorp_contract = deps.api.canonical_address(&capacorp_contract)?;
    config.capa_contract = deps.api.canonical_address(&capa_contract)?;
    config.insurance_contract = deps.api.canonical_address(&insurance_contract)?;
    store_config(&mut deps.storage, &config)?;

    Ok(HandleResponse::default())
}

pub fn update_config<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    owner_addr: Option<HumanAddr>,
) -> HandleResult {
    let mut config: Config = read_config(&deps.storage)?;

    // permission check
    if deps.api.canonical_address(&env.message.sender)? != config.owner_addr {
        return Err(StdError::unauthorized());
    }

    if let Some(owner_addr) = owner_addr {
        config.owner_addr = deps.api.canonical_address(&owner_addr)?;
    }

    store_config(&mut deps.storage, &config)?;
    Ok(HandleResponse {
        messages: vec![],
        log: vec![log("action", "update_config")],
        data: None,
    })
}

pub fn distribute<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    owner_addr: Option<HumanAddr>,
) -> HandleResult {
    let config: Config = read_config(&deps.storage)?;

    // permission check
    if deps.api.canonical_address(&env.message.sender)? != config.owner_addr {
        return Err(StdError::unauthorized());
    }
    // TODO: DISTRIBUTE HAPPENS HERE

    Ok(HandleResponse {
        messages: vec![],
        log: vec![log("action", "distribute")],
        data: None,
    })
}


pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
    }
}

pub fn query_config<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<ConfigResponse> {
    let config: Config = read_config(&deps.storage)?;
    Ok(ConfigResponse {
         owner_addr: deps.api.human_address(&config.owner_addr)?,
         market_contract: deps.api.human_address(&config.market_contract)?,
         aterra_contract: deps.api.human_address(&config.aterra_contract)?,
         cterra_contract: deps.api.human_address(&config.cterra_contract)?,
         capacorp_contract: deps.api.human_address(&config.capacorp_contract)?,
         capa_contract: deps.api.human_address(&config.capa_contract)?,
         insurance_contract: deps.api.human_address(&config.insurance_contract)?,
         stable_denom: config.stable_denom,
    })
}

