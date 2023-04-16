#[cfg(not(feature = "library"))]
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response, Uint128};

use crate::{
    actions::math::calc_sma,
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
    let provider = PROVIDERS
        .load(deps.storage, &provider_addr)
        .unwrap_or_else(|_| vec![Asset::new(&token_addr, &timestamp)]);

    let mut provider_updated: Vec<Asset> = Vec::with_capacity(provider.len());

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
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _sender: String,
    _amount: Uint128,
    _token_addr_out: String,
) -> Result<Response, ContractError> {
    Ok(Response::new().add_attributes(vec![("action", "swap")]))
}
