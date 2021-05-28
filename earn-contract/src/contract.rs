use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::{
    from_binary, log, to_binary, Api, BankMsg, Binary, CanonicalAddr, Coin, CosmosMsg, Env, Extern,
    HandleResponse, HandleResult, HumanAddr, InitResponse, InitResult, MigrateResponse,
    MigrateResult, Querier, StdError, StdResult, Storage, Uint128, WasmMsg,
};

use crate::msg::{CountResponse, HandleMsg, InitMsg, QueryMsg};
use crate::state::{read_config, store_config, Config, State};

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
            aterra_contract,
            cterra_contract,
            capacorp_contract,
            capa_contract,
            insurance_contract,
        } => register_contracts(
            deps,
            aterra_contract,
            cterra_contract,
            capacorp_contract,
            capa_contract,
            insurance_contract,
        ),
        HandleMsg::UpdateConfig { owner_addr } => update_config(deps, env, owner_addr),
        HandleMsg::Distribute { owner_addr } => distribute(deps, env, owner_addr),
        HandleMsg::Deposit => deposit(),
        HandleMsg::Withdraw => withdraw(),
    }
}

pub fn register_contracts<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    overseer_contract: HumanAddr,
    interest_model: HumanAddr,
    distribution_model: HumanAddr,
    collector_contract: HumanAddr,
    distributor_contract: HumanAddr,
) -> HandleResult {
    let mut config: Config = read_config(&deps.storage)?;
    if config.aterra_contract != CanonicalAddr::default()
        || config.cterra_contract != CanonicalAddr::default()
        || config.capacorp_contract != CanonicalAddr::default()
        || config.capa_contract != CanonicalAddr::default()
        || config.insurance_contract != CanonicalAddr::default()
    {
        return Err(StdError::unauthorized());
    }

    config.aterra_contract = deps.api.canonical_address(&overseer_contract)?;
    config.cterra_contract = deps.api.canonical_address(&interest_model)?;
    config.capacorp_contract = deps.api.canonical_address(&distribution_model)?;
    config.capa_contract = deps.api.canonical_address(&collector_contract)?;
    config.insurance_contract = deps.api.canonical_address(&distributor_contract)?;
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
    let mut config: Config = read_config(&deps.storage)?;

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

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env};
    use cosmwasm_std::{coins, from_binary, StdError};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies(20, &[]);

        /*   let msg = InitMsg { count: 17 };
        let env = mock_env("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = init(&mut deps, env, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // it worked, let's query the state
        let res = query(&deps, QueryMsg::GetCount {}).unwrap();
        let value: CountResponse = from_binary(&res).unwrap();
        assert_eq!(17, value.count);*/
    }
}
