#[cfg(not(feature = "library"))]
use cosmwasm_std::{to_binary, Binary, Deps, Env, StdError, StdResult};
use pyth_sdk_cw::{query_price_feed, PriceIdentifier};

use crate::state::{Pyth, PYTH};

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
