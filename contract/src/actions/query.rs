#[cfg(not(feature = "library"))]
use cosmwasm_std::{Addr, Decimal, Deps, Env, Order, QuerierWrapper, StdError, StdResult, Uint128};
use cw20::{BalanceResponse, Cw20QueryMsg};
use pyth_sdk_cw::{query_price_feed, Price, PriceIdentifier};

use crate::{
    actions::math::{calc_volume_ratio, u128_to_dec},
    messages::response::Balance,
    state::{Asset, Config, Pyth, Token, CONFIG, PROVIDERS, PYTH, TOKENS},
};

pub fn query_config(deps: Deps, _env: Env) -> StdResult<Config> {
    CONFIG.load(deps.storage)
}

// apr = swap_fee_rate * swapped_in_sma * token_weight / bonded_by_all_providers
pub fn query_aprs(
    deps: Deps,
    env: Env,
    address_list: Vec<String>,
) -> StdResult<Vec<(Addr, Decimal)>> {
    const APR_LIMIT: u128 = 1_000_000;

    let timestamp = env.block.time;
    let Config { swap_fee_rate, .. } = CONFIG.load(deps.storage)?;
    let token_list = query_tokens(deps, env.clone(), address_list)?;
    let provider_list = query_providers(deps, env, vec![])?;

    // get total values for volume ratio and bonded tokens
    let mut volume_ratio_list: Vec<(Addr, Decimal)> = vec![];
    let mut volume_ratio_sum = Decimal::zero();
    let mut apr_list: Vec<(Addr, Decimal)> = vec![];

    for (token_addr, token) in token_list.clone() {
        let volume_ratio = calc_volume_ratio(
            token.bonded.1,
            token.requested.1,
            token.swapped_in.1,
            token.swapped_out.1,
            swap_fee_rate,
        )?;

        volume_ratio_list.push((token_addr, volume_ratio));
        volume_ratio_sum += volume_ratio;
    }

    for (token_addr, token) in token_list {
        let bonded_by_all_providers =
            provider_list
                .iter()
                .fold(Uint128::zero(), |acc, (_, asset_list)| {
                    let asset_default = Asset::new(&token_addr, &timestamp);

                    let asset = asset_list
                        .iter()
                        .find(|x| x.token_addr == token_addr)
                        .unwrap_or(&asset_default);

                    acc + asset.bonded
                });

        // first liquidity provider will get all
        if bonded_by_all_providers.is_zero() {
            apr_list.push((token_addr, u128_to_dec(APR_LIMIT)));
            continue;
        }

        let volume_ratio_default = (token_addr.clone(), Decimal::zero());

        let (_, volume_ratio_value) = volume_ratio_list
            .iter()
            .find(|(addr, _)| addr == &token_addr)
            .unwrap_or(&volume_ratio_default);

        let token_weight = volume_ratio_value / volume_ratio_sum;

        // token.swapped_in.1 is (1 - swap_fee_rate) * swapped_in then
        // swap_fee_rate * swapped_in = swap_fee_rate * token.swapped_in.1 / (1 - swap_fee_rate)
        let apr = swap_fee_rate * u128_to_dec(token.swapped_in.1) * token_weight
            / ((Decimal::one() - swap_fee_rate) * bonded_by_all_providers);

        apr_list.push((
            token_addr,
            apr.clamp(Decimal::zero(), u128_to_dec(APR_LIMIT)),
        ));
    }

    Ok(apr_list)
}

pub fn query_providers(
    deps: Deps,
    _env: Env,
    address_list: Vec<String>,
) -> StdResult<Vec<(Addr, Vec<Asset>)>> {
    let mut res: Vec<(Addr, Vec<Asset>)> = vec![];

    for (addr, provider) in PROVIDERS
        .range(deps.storage, None, None, Order::Ascending)
        .flatten()
    {
        if address_list.is_empty() || address_list.contains(&addr.to_string()) {
            res.push((addr, provider));
        }
    }

    Ok(res)
}

pub fn query_tokens(
    deps: Deps,
    _env: Env,
    address_list: Vec<String>,
) -> StdResult<Vec<(Addr, Token)>> {
    let mut res: Vec<(Addr, Token)> = vec![];

    for (addr, token) in TOKENS
        .range(deps.storage, None, None, Order::Ascending)
        .flatten()
    {
        if address_list.is_empty() || address_list.contains(&addr.to_string()) {
            res.push((addr, token));
        }
    }

    Ok(res)
}

pub fn query_balances(deps: Deps, env: Env, address_list: Vec<String>) -> StdResult<Vec<Balance>> {
    let mut response_list: Vec<Balance> = vec![];

    for (addr, _token) in query_tokens(deps, env.clone(), address_list)? {
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

pub fn query_prices(
    deps: Deps,
    env: Env,
    address_list: Vec<String>,
) -> StdResult<Vec<(Addr, Decimal)>> {
    let Config { price_age, .. } = CONFIG.load(deps.storage)?;
    let mut price_list: Vec<(Addr, Decimal)> = vec![];

    for (addr, token) in query_tokens(deps, env.clone(), address_list)? {
        let price_feed_id_hex = &token.price_feed_id_str[2..];

        let price_feed_id = PriceIdentifier::from_hex(price_feed_id_hex)
            .map_err(|_| StdError::generic_err("Price feed is not found!"))?;

        let Pyth { pyth_contract_addr } = PYTH.load(deps.storage)?;

        let price_feed_response =
            query_price_feed(&deps.querier, pyth_contract_addr, price_feed_id)?;

        let Price { price, expo, .. } = price_feed_response
            .price_feed
            .get_price_no_older_than(env.block.time.seconds() as i64, price_age.u128() as u64)
            .ok_or_else(|| StdError::generic_err("Price is not available!"))?;

        let res = u128_to_dec(price as u128) / u128_to_dec((10u32).pow((-expo) as u32) as u128);

        price_list.push((addr, res));
    }

    Ok(price_list)
}
