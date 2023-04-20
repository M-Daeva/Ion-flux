#[cfg(not(feature = "library"))]
use cosmwasm_std::{
    to_binary, Addr, Decimal, DepsMut, Env, MessageInfo, Response, Uint128, WasmMsg,
};

use cw20::Cw20ExecuteMsg;

use crate::{
    actions::{
        math::{calc_sma, calc_volume_ratio, str_to_dec, uint128_to_dec},
        query::{query_price, query_providers, query_tokens},
    },
    error::ContractError,
    state::{Asset, Config, Sample, Token, CONFIG, PROVIDERS, TOKENS},
};

pub fn deposit(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    sender: String,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let provider_addr = deps.api.addr_validate(&sender)?;
    let token_addr = info.sender;
    let timestamp = env.block.time;
    let Config { window, .. } = CONFIG.load(deps.storage)?;

    // check if token is supported
    TOKENS
        .load(deps.storage, &token_addr)
        .map_err(|_| ContractError::TokenIsNotFound {})?;

    // check if provider exists or create new one
    let mut provider = PROVIDERS
        .load(deps.storage, &provider_addr)
        .unwrap_or_else(|_| vec![Asset::new(&token_addr, &timestamp)]);

    // if provider has no asset add it to list
    if !provider.iter().any(|x| x.token_addr == token_addr) {
        provider.push(Asset::new(&token_addr, &timestamp));
    };

    let mut provider_updated: Vec<Asset> = vec![];

    for asset in provider.iter() {
        let mut is_unbonding_counter_ready = false;
        let mut is_bonded_updated = false;

        let Asset {
            mut unbonded,
            mut requested,
            mut bonded,
            ..
        } = asset;

        // update provider token data
        if !asset.requested.is_zero() && (asset.counter <= timestamp) {
            is_unbonding_counter_ready = true;

            unbonded = unbonded
                .checked_add(requested)
                .map_err(|e| ContractError::CustomError { val: e.to_string() })?;

            requested = Uint128::zero();
        }

        if asset.token_addr == token_addr {
            is_bonded_updated = true;

            bonded = bonded
                .checked_add(amount)
                .map_err(|e| ContractError::CustomError { val: e.to_string() })?;
        };

        // update global token data
        TOKENS.update(
            deps.storage,
            &token_addr,
            |some_token| -> Result<Token, ContractError> {
                let token = some_token.ok_or(ContractError::TokenIsNotFound {})?;

                let Token {
                    unbonded: mut unbonded_sma,
                    requested: mut requested_sma,
                    bonded: mut bonded_sma,
                    ..
                } = token.clone();

                if is_unbonding_counter_ready {
                    unbonded_sma =
                        calc_sma(&token.unbonded.0, &Sample::new(unbonded, timestamp), window)?;
                    requested_sma = calc_sma(
                        &token.requested.0,
                        &Sample::new(requested, timestamp),
                        window,
                    )?;
                };

                if is_bonded_updated {
                    bonded_sma =
                        calc_sma(&token.bonded.0, &Sample::new(bonded, timestamp), window)?;
                };

                Ok(Token {
                    unbonded: unbonded_sma,
                    requested: requested_sma,
                    bonded: bonded_sma,
                    ..token
                })
            },
        )?;

        provider_updated.push(Asset {
            unbonded,
            requested,
            bonded,
            ..asset.to_owned()
        });
    }

    PROVIDERS.save(deps.storage, &provider_addr, &provider_updated)?;

    Ok(Response::new().add_attributes(vec![("action", "deposit")]))
}

pub fn swap(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    sender: String,
    amount_in: Uint128,
    token_out_addr: String,
) -> Result<Response, ContractError> {
    let token_in_addr = info.sender;
    let token_out_addr = deps.api.addr_validate(&token_out_addr)?;
    let token_in = TOKENS
        .load(deps.storage, &token_in_addr)
        .map_err(|_| ContractError::TokenIsNotFound {})?;
    let token_out = TOKENS
        .load(deps.storage, &token_out_addr)
        .map_err(|_| ContractError::TokenIsNotFound {})?;

    let token_in_price = query_price(deps.as_ref(), env.clone(), token_in.price_feed_id_str)?;
    let token_out_price = query_price(deps.as_ref(), env.clone(), token_out.price_feed_id_str)?;

    let user_addr = deps.api.addr_validate(&sender)?;
    let timestamp = env.block.time;
    let Config {
        swap_fee_rate,
        window,
        ..
    } = CONFIG.load(deps.storage)?;
    let provider_list = query_providers(deps.as_ref(), env.clone(), None)?;
    let token_list = query_tokens(deps.as_ref(), env, None)?;

    // distribute rewards to providers
    let swap_fee = swap_fee_rate * uint128_to_dec(amount_in);

    // get total values for volume ratio and bonded tokens
    let mut volume_ratio_list: Vec<(Addr, Decimal)> = vec![];
    let mut volume_ratio_sum = Decimal::zero();
    let mut bonded_total: Vec<(Addr, Uint128)> = vec![];

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

        let volume_ratio = calc_volume_ratio(
            token.bonded.1,
            token.unbonded.1,
            token.requested.1,
            token.swapped_in.1,
            token.swapped_out.1,
            swap_fee_rate,
        )?;

        volume_ratio_list.push((token_addr.clone(), volume_ratio));
        volume_ratio_sum += volume_ratio;

        bonded_total.push((token_addr, bonded_by_all_providers));
    }

    // calc rewards for each provider
    for (provider_addr, asset_list) in provider_list {
        let mut provider_power = Decimal::zero();

        for asset in asset_list {
            // calc allocation
            if asset.bonded.is_zero() {
                continue;
            }

            let bonded_default = (asset.token_addr.clone(), Uint128::zero());

            let (_, bonded_total_amount) = bonded_total
                .iter()
                .find(|(addr, _)| addr == &asset.token_addr)
                .unwrap_or(&bonded_default);

            let allocation = if bonded_total_amount.is_zero() {
                Decimal::one()
            } else {
                uint128_to_dec(asset.bonded) / uint128_to_dec(*bonded_total_amount)
            };

            let volume_ratio_default = (asset.token_addr.clone(), Decimal::zero());

            let (_, volume_ratio_value) = volume_ratio_list
                .iter()
                .find(|(addr, _)| addr == &asset.token_addr)
                .unwrap_or(&volume_ratio_default);

            let token_weight = volume_ratio_value / volume_ratio_sum;
            let asset_power = allocation * token_weight;

            provider_power += asset_power;
        }

        let provider_rewards = (provider_power * swap_fee).to_uint_floor();

        // update rewards field
        PROVIDERS.update(
            deps.storage,
            &provider_addr,
            |some_asset_list| -> Result<Vec<Asset>, ContractError> {
                let mut list = some_asset_list.ok_or(ContractError::TokenIsNotFound {})?;

                if let Some(asset_in) = list
                    .iter_mut()
                    .find(|asset| asset.token_addr == token_in_addr)
                {
                    asset_in.rewards = provider_rewards;
                } else {
                    let mut asset_in = Asset::new(&token_in_addr, &timestamp);
                    asset_in.rewards = provider_rewards;
                    list.push(asset_in);
                }

                Ok(list)
            },
        )?;
    }

    // update sma
    let amount_in_clean = amount_in - swap_fee.to_uint_ceil();

    // TODO: rewrite using price_basket
    let cost_in = str_to_dec(
        &token_in_price
            .price_feed
            .get_price_unchecked()
            .price
            .to_string(),
    ) * uint128_to_dec(amount_in_clean);

    let amount_out = (cost_in
        / str_to_dec(
            &token_out_price
                .price_feed
                .get_price_unchecked()
                .price
                .to_string(),
        ))
    .to_uint_floor();

    // swapped_in
    TOKENS.update(
        deps.storage,
        &token_in_addr,
        |some_token| -> Result<Token, ContractError> {
            let token = some_token.ok_or(ContractError::TokenIsNotFound {})?;

            Ok(Token {
                swapped_in: calc_sma(
                    &token.swapped_in.0,
                    &Sample::new(amount_in_clean, timestamp),
                    window,
                )?,
                ..token
            })
        },
    )?;

    // swapped_out
    TOKENS.update(
        deps.storage,
        &token_out_addr,
        |some_token| -> Result<Token, ContractError> {
            let token = some_token.ok_or(ContractError::TokenIsNotFound {})?;

            Ok(Token {
                swapped_out: calc_sma(
                    &token.swapped_out.0,
                    &Sample::new(amount_out, timestamp),
                    window,
                )?,
                ..token
            })
        },
    )?;

    // send token
    let cw_send_msg = Cw20ExecuteMsg::Transfer {
        recipient: user_addr.to_string(),
        amount: amount_out,
    };

    let msg = WasmMsg::Execute {
        contract_addr: token_out_addr.to_string(),
        msg: to_binary(&cw_send_msg)?,
        funds: vec![],
    };

    Ok(Response::new()
        .add_message(msg)
        .add_attributes(vec![("action", "swap")]))
}
