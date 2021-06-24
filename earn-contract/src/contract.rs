use crate::deposit::{deposit, redeem_stable};
use crate::msg::{ConfigResponse, HandleMsg, InitMsg, QueryMsg, RedeemStableHookMsg, TokenInfoResponse};
use crate::querier::{
    calculate_profit, query_capacorp_all_accounts, query_capapult_exchange_rate,
    query_dashboard,  query_token_balance, 
};
use crate::state::{
    read_config, read_profit, store_config, store_profit, store_state, Config, State,
};
use cosmwasm_storage::to_length_prefixed;
use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::{
    from_binary, log, to_binary, Api, BankMsg, Binary, CanonicalAddr, Coin, CosmosMsg, Env, Extern,
    HandleResponse, HandleResult, HumanAddr, InitResponse, InitResult, LogAttribute, Querier,
    StdError, StdResult, Storage, Uint128, QueryRequest, WasmQuery
};
use cw20::{Cw20ReceiveMsg};

pub const _1M_: u128 = 1000000;
pub const INITIAL_DEPOSIT_AMOUNT: u128 = 100000000;

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
    store_profit(&mut deps.storage, &Uint256::zero())?;

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
            deps,
            env,
            market_contract,
            aterra_contract,
            cterra_contract,
            capacorp_contract,
            capa_contract,
            insurance_contract,
        ),
        HandleMsg::UpdateConfig { owner_addr } => update_config(deps, env, owner_addr),
        HandleMsg::Distribute {} => distribute(deps, env),
     //   HandleMsg::Fees {} => pay_fees(deps, env),
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

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::ExchangeRate {} => to_binary(&query_capapult_exchange_rate(deps)?),
        QueryMsg::Dashboard  {} => to_binary(&query_dashboard(deps)),
     /*   QueryMsg::TokenBalance {
            contract_addr,
            account_addr,
        } => to_binary(&query_token_balance(
            deps,
            &HumanAddr::from(contract_addr),
            &HumanAddr::from(account_addr),
        )),*/
        QueryMsg::CorpAccounts {} => to_binary(&query_capacorp_all_accounts(deps)),
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

/*
fn calculate_fees(total_ust_amount: Uint128) -> StdResult<Uint128> {
    // Deal with fees. UST will be consumed by the earn contract, check that we have always enough
    let mut fees: Uint128 = Uint128::zero();
    let minimum_balance_for_fees = Uint128::from(100 * _1M_);
    if total_ust_amount < minimum_balance_for_fees {
        // redeem some profit for contract
        let delta = minimum_balance_for_fees.sub(Uint128::from(total_ust_amount))?;
        if delta >= Uint128::from(10 * _1M_) {
            fees = delta;
        }
    }
    Ok(fees)
}
pub fn pay_fees<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
) -> HandleResult {
    let config: Config = read_config(&deps.storage)?;

    // permission check
    if deps.api.canonical_address(&env.message.sender)? != config.owner_addr {
        return Err(StdError::unauthorized());
    }

    // Load anchor token exchange rate with updated state
    let total_ust_amount = deps.querier.query_balance(&env.contract.address, "uusd")?;
    let fees = calculate_fees(total_ust_amount.amount)?;

    if fees == Uint128::zero() {
        return Err(StdError::GenericErr {
            msg: String::from("No fees needed now"),
            backtrace: None,
        });
    }

    let profit = calculate_profit(
        deps,
        &env.contract.address,
        &deps.api.human_address(&config.aterra_contract)?,
    )?;

    if fees > profit.into() {
        return Err(StdError::GenericErr {
            msg: String::from("Not enough profit to cover fees right now"),
            backtrace: None,
        });
    }
    let mut messages: Vec<CosmosMsg> = Vec::new();
    let mut logs: Vec<LogAttribute> = Vec::new();

    let exchange_rate: Decimal256 = query_exchange_rate(deps)?;
    let redeem_amount = Uint256::from(fees) / exchange_rate;

    let redeem_stable = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: deps.api.human_address(&config.aterra_contract)?,
        send: vec![],
        msg: to_binary(&Cw20HandleMsg::Send {
            contract: deps.api.human_address(&config.market_contract)?,
            amount: redeem_amount.into(),
            msg: Some(to_binary(&RedeemStableHookMsg::RedeemStable {})?),
        })?,
    });

    let send_earn_contract = CosmosMsg::Bank(BankMsg::Send {
        from_address: env.message.sender.clone(),
        to_address: env.contract.address.clone(),
        amount: vec![Coin {
            denom: config.stable_denom.clone(),
            amount: fees.into(),
        }],
    });

    messages.push(redeem_stable);
    messages.push(send_earn_contract);
    logs.push(log("redeem_amount", redeem_amount.clone()));
    logs.push(log("fees", fees.clone()));
    Ok(HandleResponse {
        messages: messages,
        log: logs,
        data: None,
    })
}*/

fn transfer_capacorp<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    config: Config,
    profit: Uint256,
) -> HandleResult {
    let mut messages: Vec<CosmosMsg> = Vec::new();
    let mut logs: Vec<LogAttribute> = Vec::new();
    logs.push(log("action", "distribute"));

    let stake_holders = query_capacorp_all_accounts(deps)?;

    for stake_holder in stake_holders {
        let percent = query_token_balance(
            deps,
            &deps.api.human_address(&config.capacorp_contract)?,
            &stake_holder,
        )?;
        let share = Uint256::from(profit * percent) * Decimal256::from_ratio(1, 100);

        messages.push(CosmosMsg::Bank(BankMsg::Send {
            from_address: env.contract.address.clone(),
            to_address: stake_holder.clone(),
            amount: vec![Coin {
                denom: config.stable_denom.clone(),
                amount: share.into(),
            }],
        }));
        let share_str: String = share.into();
        logs.push(log(stake_holder.clone().as_str(), share_str));
    }
    let total_profit = read_profit(&deps.storage)?;
    let total_profit = total_profit + profit;
    store_profit(&mut deps.storage, &total_profit)?;
    Ok(HandleResponse {
        messages: messages,
        log: logs,
        data: None,
    })
}

pub fn distribute<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
) -> HandleResult {
    let config: Config = read_config(&deps.storage)?;

    // permission check
    if deps.api.canonical_address(&env.message.sender)? != config.owner_addr {
        return Err(StdError::unauthorized());
    }

    let res: Binary = deps.querier.query(&QueryRequest::Wasm(WasmQuery::Raw {
        contract_addr: deps.api.human_address(&config.cterra_contract)?,
        key: Binary::from(to_length_prefixed(b"token_info")),
    }))?;

    let token_info: TokenInfoResponse = from_binary(&res)?;
    let cust_total_supply = Uint256::from(token_info.total_supply);

    let profit = calculate_profit(
        deps,
        &env.contract.address,
        &deps.api.human_address(&config.aterra_contract)?,
        cust_total_supply
    )?;

    // TODO: once tested take profit when at least there is at least 100 USD of profit
    // DURING TEST, profit taking can occur with only 0.01 UST
    if profit < Uint256::from(/* INITIAL_DEPOSIT_AMOUNT * */ _1M_ / 100) {
        return Err(StdError::GenericErr {
            msg: String::from(format!("Too little profit to distribute: {}", profit)),
            backtrace: None,
        });
    }

    // TODO INSURANCE HERE

    // TODO CAPACORP DISTRIBUTION
    let response = transfer_capacorp(deps, env, config, profit)?;
    Ok(response)
}
