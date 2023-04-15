#[cfg(not(feature = "library"))]
use cosmwasm_std::{to_binary, Binary, Deps, Env, StdError, StdResult};
use pyth_sdk_cw::{query_price_feed, PriceIdentifier};

use crate::{
    error::{to_std_err, ContractError},
    state::{Pyth, PROVIDERS, PYTH},
};

pub fn query_provider(deps: Deps, _env: Env, address: String) -> StdResult<Binary> {
    let provider_addr = deps.api.addr_validate(&address)?;
    let provider = PROVIDERS
        .load(deps.storage, &provider_addr)
        .map_err(|_| to_std_err(ContractError::ProviderIsNotFound {}))?;

    to_binary(&provider)
}

pub fn query_tokens(deps: Deps, _env: Env) -> StdResult<Binary> {
    unimplemented!()
}

pub fn query_balances(deps: Deps, _env: Env) -> StdResult<Binary> {
    unimplemented!()
}

pub fn query_price(deps: Deps, _env: Env, price_feed_id_str: String) -> StdResult<Binary> {
    let price_feed_id_hex = &price_feed_id_str[2..];

    let price_feed_id = match PriceIdentifier::from_hex(price_feed_id_hex) {
        Ok(x) => x,
        Err(e) => Err(StdError::GenericErr { msg: e.to_string() })?,
    };

    let Pyth { pyth_contract_addr } = PYTH.load(deps.storage)?;

    let price_feed_response = query_price_feed(&deps.querier, pyth_contract_addr, price_feed_id)?;

    to_binary(&price_feed_response)
}
