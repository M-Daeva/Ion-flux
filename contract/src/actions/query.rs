#[cfg(not(feature = "library"))]
use cosmwasm_std::{Addr, Deps, Env, Order, QuerierWrapper, StdError, StdResult};
use cw20::{BalanceResponse, Cw20QueryMsg};
use pyth_sdk_cw::{query_price_feed, PriceFeedResponse, PriceIdentifier};

use crate::{
    error::{to_std_err, ContractError},
    messages::response::Balance,
    state::{Asset, Pyth, Token, PROVIDERS, PYTH, TOKENS},
};

// TODO: add queryApr(token_addr: &Addr) -> Uint128 and queryAprList() -> Vec<Addr, Uint128>
// apr = swap_fee_rate * swapped_in_sma * token_weight / token_bonded_total

pub fn query_providers(
    deps: Deps,
    _env: Env,
    address: Option<String>,
) -> StdResult<Vec<(Addr, Vec<Asset>)>> {
    match address {
        Some(x) => {
            let provider_addr = deps.api.addr_validate(&x)?;
            let provider = PROVIDERS
                .load(deps.storage, &provider_addr)
                .map_err(|_| to_std_err(ContractError::ProviderIsNotFound {}))?;

            Ok(vec![(provider_addr, provider)])
        }
        _ => {
            let providers: Vec<(Addr, Vec<Asset>)> = PROVIDERS
                .range(deps.storage, None, None, Order::Ascending)
                .map(|x| x.unwrap())
                .collect();

            Ok(providers)
        }
    }
}

pub fn query_tokens(
    deps: Deps,
    _env: Env,
    address: Option<String>,
) -> StdResult<Vec<(Addr, Token)>> {
    match address {
        Some(x) => {
            let token_addr = deps.api.addr_validate(&x)?;
            let token = TOKENS
                .load(deps.storage, &token_addr)
                .map_err(|_| to_std_err(ContractError::TokenIsNotFound {}))?;

            Ok(vec![(token_addr, token)])
        }
        _ => {
            let tokens: Vec<(Addr, Token)> = TOKENS
                .range(deps.storage, None, None, Order::Ascending)
                .map(|x| x.unwrap())
                .collect();

            Ok(tokens)
        }
    }
}

pub fn query_balances(deps: Deps, env: Env, address: Option<String>) -> StdResult<Vec<Balance>> {
    let mut response_list: Vec<Balance> = vec![];

    for (addr, _token) in query_tokens(deps, env.clone(), address)? {
        let msg = Cw20QueryMsg::Balance {
            address: env.contract.address.to_string(),
        };

        if let Ok(res) =
            QuerierWrapper::query_wasm_smart::<BalanceResponse>(&deps.querier, addr.clone(), &msg)
        {
            response_list.push(Balance {
                token_addr: addr,
                amount: res.balance,
            });
        }
    }

    Ok(response_list)
}

// TODO: use price_basket
pub fn query_price(
    deps: Deps,
    _env: Env,
    price_feed_id_str: String,
) -> StdResult<PriceFeedResponse> {
    let price_feed_id_hex = &price_feed_id_str[2..];

    let price_feed_id = match PriceIdentifier::from_hex(price_feed_id_hex) {
        Ok(x) => x,
        Err(e) => Err(StdError::GenericErr { msg: e.to_string() })?,
    };

    let Pyth { pyth_contract_addr } = PYTH.load(deps.storage)?;

    let price_feed_response = query_price_feed(&deps.querier, pyth_contract_addr, price_feed_id)?;

    Ok(price_feed_response)
}
