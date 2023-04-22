use cosmwasm_std::CosmosMsg;
#[cfg(not(feature = "library"))]
use cosmwasm_std::{to_binary, Decimal, DepsMut, Env, MessageInfo, Response, Uint128, WasmMsg};

use cw20::Cw20ExecuteMsg;

use crate::{
    actions::math::calc_sma,
    error::ContractError,
    state::{Asset, Config, Sample, Token, CONFIG, PROVIDERS, TOKENS},
};

#[allow(clippy::too_many_arguments)]
pub fn update_config(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    admin: Option<String>,
    swap_fee_rate: Option<Decimal>,
    window: Option<Uint128>,
    unbonding_period: Option<Uint128>,
    price_age: Option<Uint128>,
) -> Result<Response, ContractError> {
    CONFIG.update(
        deps.storage,
        |mut config| -> Result<Config, ContractError> {
            if info.sender != config.admin {
                Err(ContractError::Unauthorized {})?;
            }

            if let Some(x) = admin {
                config.admin = deps.api.addr_validate(&x)?;
            }

            if let Some(x) = swap_fee_rate {
                config.swap_fee_rate = x;
            }

            if let Some(x) = window {
                config.window = x;
            }

            if let Some(x) = unbonding_period {
                config.unbonding_period = x;
            }

            if let Some(x) = price_age {
                config.price_age = x;
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
        Err(ContractError::Unauthorized {})?;
    }

    let token_addr = deps.api.addr_validate(&token_addr)?;

    // check if token exists or create new one
    let token = TOKENS
        .load(deps.storage, &token_addr)
        .unwrap_or_else(|_| Token::new(&symbol, &price_feed_id_str));

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
    let provider_addr = info.sender;
    let token_addr = deps.api.addr_validate(&token_addr)?;
    let timestamp = env.block.time;
    let Config {
        window,
        unbonding_period,
        ..
    } = CONFIG.load(deps.storage)?;

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

    let mut provider_updated: Vec<Asset> = vec![];

    for asset in provider.iter() {
        let mut is_bonded_updated = false;

        let Asset {
            mut unbonded,
            mut requested,
            mut bonded,
            mut counter,
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
                .checked_sub(amount)
                .map_err(|_| ContractError::WithdrawAmountIsExceeded {})?;

            requested = requested
                .checked_add(amount)
                .map_err(|e| ContractError::CustomError { val: e.to_string() })?;

            counter = timestamp.plus_nanos(unbonding_period.u128() as u64);
        };

        provider_updated.push(Asset {
            unbonded,
            requested,
            bonded,
            counter,
            ..asset.to_owned()
        });

        // update global token data
        TOKENS.update(
            deps.storage,
            &token_addr,
            |some_token| -> Result<Token, ContractError> {
                let token = some_token.ok_or(ContractError::TokenIsNotFound {})?;

                let Token {
                    requested: mut requested_sma,
                    ..
                } = token.clone();

                // unbond increases APR
                if is_bonded_updated {
                    requested_sma =
                        calc_sma(&token.requested.0, &Sample::new(amount, timestamp), window)?;
                };

                Ok(Token {
                    requested: requested_sma,
                    ..token
                })
            },
        )?;
    }

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
    let provider_addr = info.sender;
    let token_addr = deps.api.addr_validate(&token_addr)?;
    let timestamp = env.block.time;

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
    let mut provider_updated: Vec<Asset> = vec![];

    for asset in provider.iter() {
        let Asset {
            mut unbonded,
            mut requested,
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

        // remove asset from list if there are no balances
        if unbonded.is_zero()
            && requested.is_zero()
            && asset.bonded.is_zero()
            && asset.rewards.is_zero()
        {
            continue;
        }

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

pub fn claim(_deps: DepsMut, _env: Env, _info: MessageInfo) -> Result<Response, ContractError> {
    unimplemented!()
}

// TODO: check if it must be in receive.rs
pub fn swap_and_claim(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _token_addr: String,
) -> Result<Response, ContractError> {
    // let token_addr = deps.api.addr_validate(&token_addr)?;
    unimplemented!()
}
