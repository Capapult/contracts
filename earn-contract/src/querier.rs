use cosmwasm_bignumber::{Decimal256, Uint256};

use crate::math::*;
use crate::msg::{QueryMsg, QueryStateMsg, AllAccountsResponse, MarketStateResponse,  TokenInfoResponse};
use crate::state::{read_config, Config};
use cosmwasm_std::{
    from_binary, to_binary, Api, Binary,  Extern, HumanAddr, Querier, QueryRequest,
     StdResult, Storage, Uint128, WasmQuery,
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
        Ok(_x) => Ok(_x.prev_exchange_rate),
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
        Ok(_x) => Ok(ExchangeRate::capapult_exchange_rate(_x.prev_exchange_rate)?),
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

pub fn query_supply<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    contract_addr: &HumanAddr,
) -> StdResult<Uint256> {
    // load price form the oracle
    let res: Binary = deps.querier.query(&QueryRequest::Wasm(WasmQuery::Raw {
        contract_addr: HumanAddr::from(contract_addr),
        key: Binary::from(to_length_prefixed(b"token_info")),
    }))?;

    let token_info: TokenInfoResponse = from_binary(&res)?;
    Ok(Uint256::from(token_info.total_supply))
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
