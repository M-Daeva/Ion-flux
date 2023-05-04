#[cfg(not(feature = "library"))]
use cosmwasm_std::{to_binary, Decimal, DepsMut, Env, MessageInfo, Response, Uint128, WasmMsg};

use cw20::Cw20ExecuteMsg;

use crate::{
    actions::{
        math::{calc_provider_rewards, calc_sma, u128_to_dec},
        query::{query_prices, query_prices_mocked, query_providers, query_tokens},
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
        let mut is_bonded_updated = false;

        let Asset {
            mut unbonded,
            mut requested,
            mut bonded,
            ..
        } = asset;

        // update provider token data
        if !asset.requested.is_zero() && (asset.counter <= timestamp) {
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

        provider_updated.push(Asset {
            unbonded,
            requested,
            bonded,
            ..asset.to_owned()
        });

        // update global token data
        TOKENS.update(
            deps.storage,
            &token_addr,
            |some_token| -> Result<Token, ContractError> {
                let token = some_token.ok_or(ContractError::TokenIsNotFound {})?;

                let Token {
                    bonded: mut bonded_sma,
                    ..
                } = token.clone();

                // deposit decreases APR
                if is_bonded_updated {
                    bonded_sma =
                        calc_sma(&token.bonded.0, &Sample::new(amount, timestamp), window)?;
                };

                Ok(Token {
                    bonded: bonded_sma,
                    ..token
                })
            },
        )?;
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
    let token_in_addr = info.sender.clone();
    let token_out_addr = deps.api.addr_validate(&token_out_addr)?;

    let price_list = if env.block.chain_id != CONFIG.load(deps.storage)?.get_chain_id() {
        query_prices(deps.as_ref(), env.clone(), vec![])
    } else {
        query_prices_mocked(deps.as_ref(), env.clone(), vec![])
    }
    .map_err(|_| ContractError::NoPrices {})?;

    let (_, token_in_price) = price_list
        .iter()
        .find(|(addr, _price)| addr == &token_in_addr)
        .ok_or(ContractError::TokenIsNotFound {})?;

    let (_, token_out_price) = price_list
        .iter()
        .find(|(addr, _price)| addr == &token_out_addr)
        .ok_or(ContractError::TokenIsNotFound {})?;

    swap_accepting_prices(
        deps,
        env,
        info,
        sender,
        amount_in,
        token_out_addr.to_string(),
        *token_in_price,
        *token_out_price,
    )
}

#[allow(clippy::too_many_arguments)]
fn swap_accepting_prices(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    sender: String,
    amount_in: Uint128,
    token_out_addr: String,
    token_in_price: Decimal,
    token_out_price: Decimal,
) -> Result<Response, ContractError> {
    let token_in_addr = info.sender;
    let token_out_addr = deps.api.addr_validate(&token_out_addr)?;

    if token_in_addr == token_out_addr {
        Err(ContractError::SameTokens {})?
    }

    let user_addr = deps.api.addr_validate(&sender)?;
    let timestamp = env.block.time;
    let Config {
        swap_fee_rate,
        window,
        ..
    } = CONFIG.load(deps.storage)?;
    let provider_list = query_providers(deps.as_ref(), env.clone(), vec![])?;
    let token_list = query_tokens(deps.as_ref(), env, vec![])?;

    // distribute rewards to providers
    let swap_fee = swap_fee_rate * u128_to_dec(amount_in);
    let amount_in_clean = amount_in - swap_fee.to_uint_ceil();

    let (provider_rewards_list, amount_out) = calc_provider_rewards(
        amount_in,
        token_in_price,
        token_out_price,
        swap_fee_rate,
        provider_list,
        token_list,
    )?;

    // update assets for each provider
    for (provider_addr, provider_rewards) in provider_rewards_list {
        PROVIDERS.update(
            deps.storage,
            &provider_addr,
            |some_asset_list| -> Result<Vec<Asset>, ContractError> {
                let list = some_asset_list.ok_or(ContractError::TokenIsNotFound {})?;
                let mut list_updated: Vec<Asset> = vec![];

                // update unbonded
                for mut asset in list {
                    if !asset.requested.is_zero() && (asset.counter <= timestamp) {
                        asset.unbonded = asset
                            .unbonded
                            .checked_add(asset.requested)
                            .map_err(|e| ContractError::CustomError { val: e.to_string() })?;

                        asset.requested = Uint128::zero();
                    }

                    list_updated.push(asset);
                }

                // update rewards
                if let Some(asset_in) = list_updated
                    .iter_mut()
                    .find(|asset| asset.token_addr == token_in_addr)
                {
                    asset_in.rewards = provider_rewards;
                } else {
                    let mut asset_in = Asset::new(&token_in_addr, &timestamp);
                    asset_in.rewards = provider_rewards;
                    list_updated.push(asset_in);
                }

                Ok(list_updated)
            },
        )?;
    }

    // update sma values
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
