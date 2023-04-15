use cosmwasm_std::CosmosMsg;
#[cfg(not(feature = "library"))]
use cosmwasm_std::{to_binary, Decimal, DepsMut, Env, MessageInfo, Response, Uint128, WasmMsg};

use cw20::Cw20ExecuteMsg;

use crate::{
    actions::math::calc_sma,
    error::ContractError,
    state::{Asset, Config, Sample, Token, CONFIG, PROVIDERS, TOKENS},
};

pub fn update_config(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    admin: Option<String>,
    swap_fee_rate: Option<Decimal>,
    window: Option<Uint128>,
    unbonding_period: Option<Uint128>,
) -> Result<Response, ContractError> {
    CONFIG.update(
        deps.storage,
        |mut config| -> Result<Config, ContractError> {
            if info.sender != config.admin {
                return Err(ContractError::Unauthorized {});
            }

            if let Some(admin) = admin {
                config = Config {
                    admin: deps.api.addr_validate(&admin)?,
                    ..config
                };
            }

            if let Some(swap_fee_rate) = swap_fee_rate {
                config = Config {
                    swap_fee_rate,
                    ..config
                };
            }

            if let Some(window) = window {
                config = Config { window, ..config };
            }

            if let Some(unbonding_period) = unbonding_period {
                config = Config {
                    unbonding_period,
                    ..config
                };
            }

            Ok(config)
        },
    )?;

    Ok(Response::new().add_attributes(vec![("action", "update_config")]))
}

pub fn update_token(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    token_addr: String,
    symbol: String,
    price_feed_id_str: String,
) -> Result<Response, ContractError> {
    if info.sender != CONFIG.load(deps.storage)?.admin {
        return Err(ContractError::Unauthorized {});
    }

    let token_addr = deps.api.addr_validate(&token_addr)?;

    // check if token exists or create new one
    let token = match TOKENS.load(deps.storage, &token_addr) {
        Ok(x) => x,
        _ => Token::new(&symbol, &price_feed_id_str),
    };

    TOKENS.save(
        deps.storage,
        &token_addr,
        &Token {
            symbol,
            price_feed_id_str,
            ..token
        },
    )?;

    Ok(Response::new().add_attributes(vec![("action", "update_token")]))
}

pub fn unbond(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token_addr: String,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let provider_addr = info.sender.clone();
    let token_addr = deps.api.addr_validate(&token_addr)?;
    let timestamp = env.block.time;
    let Config {
        window,
        unbonding_period,
        ..
    } = CONFIG.load(deps.storage)?;

    // check if token is supported
    TOKENS.load(deps.storage, &token_addr)?;

    // check if provider exists or return err
    let provider = match PROVIDERS.load(deps.storage, &provider_addr) {
        Ok(x) => x,
        _ => Err(ContractError::ProviderIsNotFound {})?,
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
                bonded -= amount;
                requested += amount;
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

                        if is_unbonding_counter_ready || is_bonded_updated {
                            requested_sma = calc_sma(
                                &token.requested.0,
                                &Sample::new(requested, timestamp),
                                window,
                            );
                        };

                        if is_unbonding_counter_ready {
                            unbonded_sma = calc_sma(
                                &token.unbonded.0,
                                &Sample::new(unbonded, timestamp),
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
                counter: timestamp.plus_nanos(unbonding_period.u128() as u64),
                ..x.to_owned()
            }
        })
        .collect();

    PROVIDERS.save(deps.storage, &provider_addr, &provider_updated)?;

    Ok(Response::new().add_attributes(vec![("action", "unbond")]))
}

pub fn withdraw(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token_addr: String,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let provider_addr = info.sender.clone();
    let token_addr = deps.api.addr_validate(&token_addr)?;
    let timestamp = env.block.time;
    let Config { window, .. } = CONFIG.load(deps.storage)?;

    // check if token is supported
    TOKENS
        .load(deps.storage, &token_addr)
        .map_err(|_| ContractError::TokenIsNotFound {})?;

    // check if provider exists or return err
    let provider = PROVIDERS
        .load(deps.storage, &provider_addr)
        .map_err(|_| ContractError::ProviderIsNotFound {})?;

    // check if provider has any funds in the app
    if provider.is_empty() {
        Err(ContractError::FundsAreNotFound {})?;
    }

    let mut msgs: Vec<CosmosMsg> = vec![];
    let mut provider_updated: Vec<Asset> = Vec::with_capacity(provider.len());

    for asset in provider.iter() {
        let mut is_unbonding_counter_ready = false;
        let mut is_unbonded_updated = false;

        let Asset {
            mut unbonded,
            mut requested,
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
            is_unbonded_updated = true;

            unbonded = unbonded
                .checked_sub(amount)
                .map_err(|_| ContractError::WithdrawAmountIsExceeded {})?;

            let cw_send_msg = Cw20ExecuteMsg::Transfer {
                recipient: provider_addr.to_string(),
                amount,
            };

            let msg = WasmMsg::Execute {
                contract_addr: token_addr.to_string(),
                msg: to_binary(&cw_send_msg)?,
                funds: vec![],
            };

            msgs.push(msg.into());
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
                    ..
                } = token.clone();

                if is_unbonding_counter_ready || is_unbonded_updated {
                    unbonded_sma =
                        calc_sma(&token.unbonded.0, &Sample::new(unbonded, timestamp), window);
                };

                if is_unbonding_counter_ready {
                    requested_sma = calc_sma(
                        &token.requested.0,
                        &Sample::new(requested, timestamp),
                        window,
                    );
                };

                Ok(Token {
                    unbonded: unbonded_sma,
                    requested: requested_sma,
                    ..token
                })
            },
        )?;

        provider_updated.push(Asset {
            unbonded,
            requested,
            ..asset.to_owned()
        });
    }

    PROVIDERS.save(deps.storage, &provider_addr, &provider_updated)?;

    Ok(Response::new()
        .add_messages(msgs)
        .add_attributes(vec![("action", "withdraw")]))
}

pub fn claim(_deps: DepsMut, _env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    unimplemented!()
}

// TODO: check if it must be in receive.rs
pub fn swap_and_claim(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    token_addr: String,
) -> Result<Response, ContractError> {
    let token_addr = deps.api.addr_validate(&token_addr)?;
    unimplemented!()
}
