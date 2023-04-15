#[cfg(not(feature = "library"))]
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response, StdError, Uint128};

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
    let token_addr = info.sender.clone();
    let timestamp = env.block.time;
    let Config { window, .. } = CONFIG.load(deps.storage)?;

    // check if token is supported
    TOKENS.load(deps.storage, &token_addr)?;

    // check if provider exists or create new one
    let provider = match PROVIDERS.load(deps.storage, &provider_addr) {
        Ok(x) => x,
        _ => vec![],
    };

    let provider_updated: Vec<Asset> = provider
        .iter()
        .map(|x| {
            let mut is_unbonding_counter_ready = false;
            let mut is_bonded_updated = false;

            let Asset {
                mut unbonded,
                mut requested,
                mut bonded,
                ..
            } = x;

            // update provider token data
            if !x.requested.is_zero() && (x.counter <= timestamp) {
                is_unbonding_counter_ready = true;
                unbonded += requested;
                requested = Uint128::zero();
            }

            if x.token_addr == token_addr {
                is_bonded_updated = true;
                bonded += amount;
            };

            // update global token data
            TOKENS
                .update(
                    deps.storage,
                    &token_addr,
                    |some_token| -> Result<Token, ContractError> {
                        let token = some_token.unwrap();
                        let Token {
                            unbonded: mut unbonded_sma,
                            requested: mut requested_sma,
                            bonded: mut bonded_sma,
                            ..
                        } = token.clone();

                        if is_unbonding_counter_ready {
                            unbonded_sma = calc_sma(
                                &token.unbonded.0,
                                &Sample::new(unbonded, timestamp),
                                window,
                            );
                            requested_sma = calc_sma(
                                &token.requested.0,
                                &Sample::new(requested, timestamp),
                                window,
                            );
                        };

                        if is_bonded_updated {
                            bonded_sma =
                                calc_sma(&token.bonded.0, &Sample::new(bonded, timestamp), window);
                        };

                        Ok(Token {
                            unbonded: unbonded_sma,
                            requested: requested_sma,
                            bonded: bonded_sma,
                            ..token
                        })
                    },
                )
                .unwrap();

            Asset {
                unbonded,
                requested,
                bonded,
                ..x.to_owned()
            }
        })
        .collect();

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
