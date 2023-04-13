#[cfg(not(feature = "library"))]
use cosmwasm_std::{to_binary, Decimal, DepsMut, Env, MessageInfo, Response, Uint128, WasmMsg};

use cw20::Cw20ExecuteMsg;

use crate::{
    error::ContractError,
    state::{Config, Token, CONFIG, TOKENS},
};

pub fn update_config(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    admin: Option<String>,
    swap_fee_rate: Option<Decimal>,
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
    _env: Env,
    info: MessageInfo,
    token_addr: String,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let token_addr = deps.api.addr_validate(&token_addr)?;
    unimplemented!()
}

// TODO: rewrite
pub fn withdraw(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    token_addr: String,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let token_addr = deps.api.addr_validate(&token_addr)?;

    let token = match TOKENS.load(deps.storage, &token_addr) {
        Ok(x) => x,
        _ => Err(ContractError::TokenIsNotFound {})?,
    };

    let cw_send_msg = Cw20ExecuteMsg::Transfer {
        recipient: info.sender.to_string(),
        amount,
    };

    let msg = WasmMsg::Execute {
        contract_addr: token_addr.to_string(),
        msg: to_binary(&cw_send_msg)?,
        funds: vec![],
    };

    Ok(Response::new()
        .add_message(msg)
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
