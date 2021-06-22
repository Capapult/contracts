use cosmwasm_bignumber::{Decimal256, Uint256};

use crate::math::*;
use crate::msg::{QueryMsg, QueryStateMsg, AllAccountsResponse, MarketStateResponse,  TokenInfoResponse};
use crate::state::{read_config, Config, read_profit};

use cosmwasm_std::{
    from_binary, to_binary, Api, Binary,  Extern, HumanAddr, Querier, QueryRequest,
     StdResult, Storage, Uint128, WasmQuery,StdError
};

use cosmwasm_storage::{to_length_prefixed};

pub fn query_exchange_rate<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<Decimal256> {
    let config: Config = read_config(&deps.storage)?;
    let market_state: StdResult<MarketStateResponse> =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: deps.api.human_address(&config.market_contract)?,
            msg: to_binary(&QueryStateMsg::State {})?,
        }));

    match market_state {
        Ok(_x) => Ok(_x.global_interest_index),
        Err(_x) => Err(_x),
    }
}

pub fn query_capapult_exchange_rate<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<Decimal256> {
    let config: Config = read_config(&deps.storage)?;
    let market_state: StdResult<MarketStateResponse> =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: deps.api.human_address(&config.market_contract)?,
            msg: to_binary(&QueryStateMsg::State {})?,
        }));

    match market_state {
        Ok(_x) => Ok(ExchangeRate::capapult_exchange_rate(_x.global_interest_index)?),
        Err(_x) => Err(_x),
    }
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

pub fn query_cust_supply<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<Uint256> {
    let config: Config = read_config(&deps.storage)?;
    // load price form the oracle
    let res: Binary = deps.querier.query(&QueryRequest::Wasm(WasmQuery::Raw {
        contract_addr: deps.api.human_address(&config.cterra_contract)?,
        key: Binary::from(to_length_prefixed(b"token_info")),
    }))?;

    let token_info: TokenInfoResponse = from_binary(&res)?;
    Ok(Uint256::from(token_info.total_supply))
}

pub fn query_cust_accounts<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<usize> {
    let config: Config = read_config(&deps.storage)?;
    // load price form the oracle
    let res: Binary = deps.querier.query(&QueryRequest::Wasm(WasmQuery::Raw {
        contract_addr: deps.api.human_address(&config.cterra_contract)?,
        key: Binary::from(to_length_prefixed(b"all_accounts")),
    }))?;

    let all_accounts: AllAccountsResponse = from_binary(&res)?;
    Ok(all_accounts.accounts.len())
}

pub fn calculate_profit<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    earn_contract: &HumanAddr,
    aterra_contract: &HumanAddr,
) -> StdResult<Uint256> {
    // Load anchor token exchange rate with updated state
    let exchange_rate: Decimal256 = query_exchange_rate(deps)?;
    let capa_exchange_rate = ExchangeRate::capapult_exchange_rate(exchange_rate)?;

    let total_aterra_amount = query_token_balance(deps, aterra_contract, earn_contract)?;
    let total_c_ust_amount = query_cust_supply(deps)?;

    let res1 = Uint256::from(total_aterra_amount) * exchange_rate;
    let res2 = Uint256::from(total_c_ust_amount) * capa_exchange_rate;
    if res1 <= res2 {
        return Err(StdError::GenericErr {
            msg: String::from("No profit to distribute"),
            backtrace: None,
        });
    }

    Ok(res1 - res2)
}


pub fn query_current_profit<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<Uint256> {
    let config: Config = read_config(&deps.storage)?;

    // TODO how to permission check with queries ?
   // if deps.api.canonical_address(&env.message.sender)? != config.owner_addr {
   //     return Err(StdError::unauthorized());
   // }

    let profit = calculate_profit(
        deps,
        &deps.api.human_address(&config.contract_addr)?,
        &deps.api.human_address(&config.aterra_contract)?,
    )?;
    Ok(profit)
}

pub fn query_total_profit<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<Uint256> {
    // TODO how to permission check with queries ?
   // if deps.api.canonical_address(&env.message.sender)? != config.owner_addr {
   //     return Err(StdError::unauthorized());
   // }
   

    let profit: Uint256 = read_profit(&deps.storage)?;

    Ok(profit)
}
pub fn query_capacorp_all_accounts<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<Vec<HumanAddr>> {
    
    let config: Config = read_config(&deps.storage)?;
    let all_accounts: StdResult<AllAccountsResponse> =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: deps.api.human_address(&config.capacorp_contract)?,
            msg: to_binary(&QueryMsg::AllAccounts {})?,
        }));

    match all_accounts {
        Ok(_x) => Ok(_x.accounts),
        Err(_x) => Err(_x),
    }
}

#[inline]
fn concat(namespace: &[u8], key: &[u8]) -> Vec<u8> {
    let mut k = namespace.to_vec();
    k.extend_from_slice(key);
    k
}
