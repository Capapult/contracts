use cosmwasm_bignumber::{Decimal256, Uint256};

use crate::math::*;
use crate::msg::{
    Account, AllAccountsResponse, DashboardResponse, MarketStateResponse, QueryStateMsg,
    TokenInfoResponse,
};
use crate::state::{read_config, read_profit, read_total_deposit, Config};

use cosmwasm_std::{
    from_binary, to_binary, Addr, Binary, Coin, Deps, QueryRequest, StdResult, Uint128, WasmQuery,
};

use cosmwasm_storage::to_length_prefixed;
use terra_cosmwasm::TerraQuerier;

pub fn query_exchange_rate(deps: Deps) -> StdResult<Decimal256> {
    let config: Config = read_config(deps.storage)?;
    let market_state: StdResult<MarketStateResponse> =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: config.market_contract,
            msg: to_binary(&QueryStateMsg::State {})?,
        }));

    match market_state {
        Ok(_x) => Ok(_x.prev_exchange_rate),
        Err(_x) => Err(_x),
    }
}

pub fn query_capapult_exchange_rate(deps: Deps) -> StdResult<Decimal256> {
    let config: Config = read_config(deps.storage)?;
    let market_state: StdResult<MarketStateResponse> =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: config.market_contract,
            msg: to_binary(&QueryStateMsg::State {})?,
        }));

    match market_state {
        Ok(_x) => Ok(ExchangeRate::capapult_exchange_rate(_x.prev_exchange_rate)?),
        Err(_x) => Err(_x),
    }
}

pub fn query_token_balance(
    deps: Deps,
    contract_addr: &Addr,
    account_addr: &Addr,
) -> StdResult<Uint256> {
    // load balance form the token contract
    let account: String = account_addr.into();
    let res: Binary = deps
        .querier
        .query(&QueryRequest::Wasm(WasmQuery::Raw {
            contract_addr: contract_addr.into(),
            key: Binary::from(concat(
                &to_length_prefixed(b"balance").to_vec(),
                &account.as_bytes(),
            )),
        }))
        .unwrap_or_else(|_| to_binary(&Uint128::zero()).unwrap());

    let balance: Uint128 = from_binary(&res)?;
    Ok(balance.into())
}

pub fn query_dashboard(deps: Deps) -> StdResult<DashboardResponse> {
    let config: Config = read_config(deps.storage)?;

    let res: Binary = deps.querier.query(&QueryRequest::Wasm(WasmQuery::Raw {
        contract_addr: config.cterra_contract.clone(),
        key: Binary::from(to_length_prefixed(b"token_info")),
    }))?;

    let token_info: TokenInfoResponse = from_binary(&res)?;
    let cust_total_supply = Uint256::from(token_info.total_supply);

    let all_accounts: AllAccountsResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: config.cterra_contract.clone(),
            msg: to_binary(&Account::AllAccounts {})?,
        }))?;

    let current_profit = calculate_profit(
        deps,
        &deps.api.addr_validate(&config.contract_addr)?,
        &deps.api.addr_validate(&config.aterra_contract)?,
        cust_total_supply,
    )?;
    let total_profit: Uint256 = read_profit(deps.storage)?;

    let mut total_value_locked: Uint256 = query_token_balance(
        deps,
        &deps.api.addr_validate(&config.aterra_contract)?,
        &deps.api.addr_validate(&config.contract_addr)?,
    )?;

    let market_state: MarketStateResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: config.market_contract,
            msg: to_binary(&QueryStateMsg::State {})?,
        }))?;

    total_value_locked = total_value_locked * market_state.prev_exchange_rate;

    let cust_nb_accounts = Uint256::from(all_accounts.accounts.len() as u128);
    let cust_total_supply = Uint256::from(token_info.total_supply);
    let mut cust_avg_balance = Decimal256::zero();
    if cust_nb_accounts > Uint256::zero() {
        cust_avg_balance = Decimal256::from_uint256(cust_total_supply)
            / Decimal256::from_uint256(cust_nb_accounts);
    }

    Ok(DashboardResponse {
        total_value_locked,
        cust_total_supply,
        cust_nb_accounts,
        cust_avg_balance,
        current_profit,
        total_profit,
    })
}

pub fn query_capacorp_all_accounts(deps: Deps) -> StdResult<Vec<Addr>> {
    let config: Config = read_config(deps.storage)?;
    let all_accounts: AllAccountsResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: config.capacorp_contract,
            msg: to_binary(&Account::AllAccounts {})?,
        }))?;

    Ok(all_accounts.accounts)
}

pub fn calculate_profit(
    deps: Deps,
    earn_contract: &Addr,
    aterra_contract: &Addr,
    total_c_ust_supply: Uint256,
) -> StdResult<Uint256> {
    // Load anchor token exchange rate with updated state
    let exchange_rate: Decimal256 = query_exchange_rate(deps)?;
    let capa_exchange_rate = ExchangeRate::capapult_exchange_rate(exchange_rate)?;

    let total_aterra_amount = query_token_balance(deps, aterra_contract, earn_contract)?;

    let res1 = total_aterra_amount * exchange_rate;
    let res2 = total_c_ust_supply * capa_exchange_rate;
    if res1 <= res2 {
        return Ok(Uint256::zero());
    }

    Ok(res1 - res2)
}

pub fn calculate_aterra_profit(
    deps: Deps,
    earn_contract: &Addr,
    aterra_contract: &Addr,
    total_c_ust_supply: Uint256,
) -> StdResult<Uint256> {
    // Load anchor token exchange rate with updated state
    let exchange_rate: Decimal256 = query_exchange_rate(deps)?;
    let capa_exchange_rate = ExchangeRate::capapult_exchange_rate(exchange_rate)?;

    let total_aterra_amount = query_token_balance(deps, aterra_contract, earn_contract)?;

    let res1 = total_aterra_amount * exchange_rate;
    let res2 = total_c_ust_supply * capa_exchange_rate;
    if res1 <= res2 {
        return Ok(Uint256::zero());
    }

    Ok((res1 - res2) / exchange_rate)
}

pub fn query_harvest_value(deps: Deps, account_addr: String) -> StdResult<Uint256> {
    let config: Config = read_config(deps.storage)?;

    let exchange_rate: Decimal256 = query_exchange_rate(deps)?;
    let capa_exchange_rate = ExchangeRate::capapult_exchange_rate(exchange_rate)?;

    let cust_balance = query_token_balance(
        deps,
        &deps.api.addr_validate(&config.cterra_contract)?,
        &deps.api.addr_validate(&account_addr)?,
    )?;
    let total_deposit = read_total_deposit(deps.storage, &account_addr)?;

    let mut amount = Uint256::from(0u128);
    let current_ust = cust_balance * capa_exchange_rate;

    if current_ust > total_deposit {
        amount = current_ust - total_deposit;
    }

    Ok(amount)
}

#[inline]
fn concat(namespace: &[u8], key: &[u8]) -> Vec<u8> {
    let mut k = namespace.to_vec();
    k.extend_from_slice(key);
    k
}

pub fn compute_tax(deps: Deps, coin: &Coin) -> StdResult<Uint256> {
    let terra_querier = TerraQuerier::new(&deps.querier);
    let tax_rate = Decimal256::from((terra_querier.query_tax_rate()?).rate);
    let tax_cap = Uint256::from((terra_querier.query_tax_cap(coin.denom.to_string())?).cap);
    let amount = Uint256::from(coin.amount);
    Ok(std::cmp::min(
        amount * (Decimal256::one() - Decimal256::one() / (Decimal256::one() + tax_rate)),
        tax_cap,
    ))
}
pub fn deduct_tax(deps: Deps, coin: Coin) -> StdResult<Coin> {
    let tax_amount = compute_tax(deps, &coin)?;
    Ok(Coin {
        denom: coin.denom,
        amount: (Uint256::from(coin.amount) - tax_amount).into(),
    })
}
