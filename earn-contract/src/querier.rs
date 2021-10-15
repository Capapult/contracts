use cosmwasm_bignumber::{Decimal256, Uint256};

use crate::math::*;
use crate::msg::{Account, DashboardResponse, MarketStateResponse, QueryStateMsg};
use crate::state::{read_config, read_profit, read_total_claim, read_total_deposit, Config};
use cw20::{AllAccountsResponse, Cw20QueryMsg, TokenInfoResponse};

use cosmwasm_std::{
    to_binary, Addr, CanonicalAddr, Coin, Deps, QueryRequest, StdResult, Uint128, WasmQuery,
};

use terra_cosmwasm::TerraQuerier;

pub fn query_exchange_rate(deps: Deps) -> StdResult<Decimal256> {
    let config: Config = read_config(deps.storage)?;
    let market_state: StdResult<MarketStateResponse> =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: deps.api.addr_humanize(&config.market_contract)?.to_string(),
            msg: to_binary(&QueryStateMsg::State {})?,
        }));

    let prev_exchange_rate = market_state?.prev_exchange_rate;
    Ok(prev_exchange_rate)
}

pub fn query_capapult_exchange_rate(deps: Deps) -> StdResult<Decimal256> {
    let config: Config = read_config(deps.storage)?;
    let market_state: StdResult<MarketStateResponse> =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: deps.api.addr_humanize(&config.market_contract)?.to_string(),
            msg: to_binary(&QueryStateMsg::State {})?,
        }));

    let exchange_rate = ExchangeRate::capapult_exchange_rate(market_state?.prev_exchange_rate)?;
    Ok(exchange_rate)
}

pub fn query_token_balance(
    deps: Deps,
    contract_addr: &Addr,
    account_addr: &Addr,
) -> StdResult<Uint256> {
    // load balance form the token contract
    let balance: Uint128 = deps
        .querier
        .query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: contract_addr.to_string(),
            msg: to_binary(&Cw20QueryMsg::Balance {
                address: account_addr.to_string(),
            })?,
        }))
        .unwrap_or_else(|_| Uint128::zero());
    Ok(balance.into())
}

pub fn query_token_supply(deps: Deps, contract_addr: Addr) -> StdResult<Uint256> {
    let token_info: TokenInfoResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: contract_addr.to_string(),
            msg: to_binary(&Cw20QueryMsg::TokenInfo {})?,
        }))?;

    Ok(Uint256::from(token_info.total_supply))
}

pub fn query_dashboard(deps: Deps) -> StdResult<DashboardResponse> {
    let config: Config = read_config(deps.storage)?;

    let cust_total_supply =
        query_token_supply(deps, deps.api.addr_humanize(&config.cterra_contract)?)?;

    let all_accounts: AllAccountsResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: deps.api.addr_humanize(&config.cterra_contract)?.to_string(),
            msg: to_binary(&Account::AllAccounts {})?,
        }))?;

    let current_profit = calculate_profit(
        deps,
        &deps.api.addr_humanize(&config.contract_addr)?,
        &deps.api.addr_humanize(&config.aterra_contract)?,
        cust_total_supply,
    )?;

    let total_profit: Uint256 = read_profit(deps.storage)?;

    let mut total_value_locked: Uint256 = query_token_balance(
        deps,
        &deps.api.addr_humanize(&config.aterra_contract)?,
        &deps.api.addr_humanize(&config.contract_addr)?,
    )?;

    let market_state: MarketStateResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: deps.api.addr_humanize(&config.market_contract)?.to_string(),
            msg: to_binary(&QueryStateMsg::State {})?,
        }))?;

    total_value_locked = total_value_locked * market_state.prev_exchange_rate;

    let cust_nb_accounts = Uint256::from(all_accounts.accounts.len() as u128);

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

pub fn query_capacorp_all_accounts(deps: Deps) -> StdResult<Vec<String>> {
    let config: Config = read_config(deps.storage)?;
    let all_accounts: AllAccountsResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: deps
                .api
                .addr_humanize(&config.capacorp_contract)?
                .to_string(),
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
        &deps.api.addr_humanize(&config.cterra_contract)?,
        &deps.api.addr_validate(&account_addr)?,
    )?;
    let account_addr_canon: CanonicalAddr = deps.api.addr_canonicalize(account_addr.as_str())?;
    let total_deposit = read_total_deposit(deps.storage, &account_addr_canon);
    let current_claim = read_total_claim(deps.storage, &account_addr_canon);

    let current_ust = cust_balance * capa_exchange_rate;
    let sum_deposit_claim = total_deposit + current_claim;

    if current_ust > sum_deposit_claim {
        return Ok(current_ust - sum_deposit_claim);
    }

    Ok(Uint256::from(0u128))
}

pub fn query_harvested_sum(deps: Deps, account_addr: String) -> StdResult<Uint256> {
    let account_addr_canon: CanonicalAddr = deps.api.addr_canonicalize(account_addr.as_str())?;
    Ok(read_total_claim(deps.storage, &account_addr_canon))
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
