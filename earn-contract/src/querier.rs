use cosmwasm_bignumber::{Decimal256, Uint256};

use crate::math::*;
use crate::msg::{
    AllAccountsResponse, DashboardResponse, MarketStateResponse, Account, QueryStateMsg, TokenInfoResponse,
};
use crate::state::{read_config, read_profit, Config};

use cosmwasm_std::{
    from_binary, to_binary, Api, Binary, Coin, Extern, HumanAddr, Querier, QueryRequest,
    StdResult, Storage, Uint128, WasmQuery,
};

use cosmwasm_storage::to_length_prefixed;
use terra_cosmwasm::TerraQuerier;

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

pub fn query_dashboard<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<DashboardResponse> {
    let config: Config = read_config(&deps.storage)?;

    let res: Binary = deps.querier.query(&QueryRequest::Wasm(WasmQuery::Raw {
        contract_addr: deps.api.human_address(&config.cterra_contract)?,
        key: Binary::from(to_length_prefixed(b"token_info")),
    }))?;

    let token_info: TokenInfoResponse = from_binary(&res)?;
    let cust_total_supply = Uint256::from(token_info.total_supply);

    let all_accounts: AllAccountsResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: deps.api.human_address(&config.cterra_contract)?,
            msg: to_binary(&Account::AllAccounts {})?,
        }))?;

    let current_profit = calculate_profit(
        deps,
        &deps.api.human_address(&config.contract_addr)?,
        &deps.api.human_address(&config.aterra_contract)?,
        cust_total_supply
    )?;
    let total_profit: Uint256 = read_profit(&deps.storage)?;

    let mut total_value_locked: Uint256 = Uint256::from(
        query_token_balance(deps, &deps.api.human_address(&config.aterra_contract)?, 
        &deps.api.human_address(&config.contract_addr)?)?
    );
    
    let market_state: MarketStateResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: deps.api.human_address(&config.market_contract)?,
            msg: to_binary(&QueryStateMsg::State {})?,
        }))?;

    total_value_locked = total_value_locked * market_state.prev_exchange_rate;

    let cust_nb_accounts = Uint256::from(all_accounts.accounts.len() as u128);
    let cust_total_supply = Uint256::from(token_info.total_supply);
    let mut cust_avg_balance = Decimal256::zero();
    if cust_nb_accounts > Uint256::zero() {
        cust_avg_balance  = Decimal256::from_uint256(cust_total_supply) /  Decimal256::from_uint256(cust_nb_accounts);
    }     

    Ok(DashboardResponse {
        total_value_locked: total_value_locked,
        cust_total_supply: cust_total_supply,
        cust_nb_accounts: cust_nb_accounts,
        cust_avg_balance: cust_avg_balance,
        current_profit: current_profit,
        total_profit: total_profit
    })
}

pub fn query_capacorp_all_accounts<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<Vec<HumanAddr>> {
    let config: Config = read_config(&deps.storage)?;
    let all_accounts: AllAccountsResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: deps.api.human_address(&config.capacorp_contract)?,
            msg: to_binary(&Account::AllAccounts {})?,
        }))?;

    Ok(all_accounts.accounts)
}

pub fn calculate_profit<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    earn_contract: &HumanAddr,
    aterra_contract: &HumanAddr,
    total_c_ust_supply: Uint256
) -> StdResult<Uint256> {
    // Load anchor token exchange rate with updated state
    let exchange_rate: Decimal256 = query_exchange_rate(deps)?;
    let capa_exchange_rate = ExchangeRate::capapult_exchange_rate(exchange_rate)?;

    let total_aterra_amount = query_token_balance(deps, aterra_contract, earn_contract)?;

    let res1 = Uint256::from(total_aterra_amount) * exchange_rate;
    let res2 = Uint256::from(total_c_ust_supply) * capa_exchange_rate;
    if res1 <= res2 {
        return Ok(Uint256::zero());
    }

    Ok(res1 - res2)
}

pub fn calculate_aterra_profit<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    earn_contract: &HumanAddr,
    aterra_contract: &HumanAddr,
    total_c_ust_supply: Uint256
) -> StdResult<Uint256> {
    // Load anchor token exchange rate with updated state
    let exchange_rate: Decimal256 = query_exchange_rate(deps)?;
    let capa_exchange_rate = ExchangeRate::capapult_exchange_rate(exchange_rate)?;

    let total_aterra_amount = query_token_balance(deps, aterra_contract, earn_contract)?;

    let res1 = Uint256::from(total_aterra_amount) * exchange_rate;
    let res2 = Uint256::from(total_c_ust_supply) * capa_exchange_rate;
    if res1 <= res2 {
        return Ok(Uint256::zero());
    }

    Ok((res1 - res2) /  exchange_rate)
}

#[inline]
fn concat(namespace: &[u8], key: &[u8]) -> Vec<u8> {
    let mut k = namespace.to_vec();
    k.extend_from_slice(key);
    k
}

pub fn compute_tax<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    coin: &Coin,
) -> StdResult<Uint256> {
    let terra_querier = TerraQuerier::new(&deps.querier);
    let tax_rate = Decimal256::from((terra_querier.query_tax_rate()?).rate);
    let tax_cap = Uint256::from((terra_querier.query_tax_cap(coin.denom.to_string())?).cap);
    let amount = Uint256::from(coin.amount);
    Ok(std::cmp::min(
        amount * (Decimal256::one() - Decimal256::one() / (Decimal256::one() + tax_rate)),
        tax_cap,
    ))
}
pub fn deduct_tax<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    coin: Coin,
) -> StdResult<Coin> {
    let tax_amount = compute_tax(deps, &coin)?;
    Ok(Coin {
        denom: coin.denom,
        amount: (Uint256::from(coin.amount) - tax_amount).into(),
    })
}
