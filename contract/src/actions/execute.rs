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

pub fn update_tokens(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    symbol: String,
    token_addr: String,
    price_feed_id_str: String,
) -> Result<Response, ContractError> {
    if info.sender != CONFIG.load(deps.storage)?.admin {
        return Err(ContractError::Unauthorized {});
    }

    TOKENS.save(
        deps.storage,
        &symbol,
        &Token {
            token_addr: deps.api.addr_validate(&token_addr)?,
            price_feed_id_str,
        },
    )?;

    Ok(Response::new().add_attributes(vec![("action", "update_tokens")]))
}

pub fn unbond(
    _deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    symbol: String,
    amount: Uint128,
) -> Result<Response, ContractError> {
    unimplemented!()
}

// TODO: rewrite
pub fn withdraw(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    symbol: String,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let token = match TOKENS.load(deps.storage, &symbol) {
        Ok(x) => x,
        _ => Err(ContractError::TokenIsNotFound {})?,
    };

    let cw_send_msg = Cw20ExecuteMsg::Transfer {
        recipient: info.sender.to_string(),
        amount,
    };

    let msg = WasmMsg::Execute {
        contract_addr: token.token_addr.to_string(),
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
    _deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    symbol: String,
) -> Result<Response, ContractError> {
    unimplemented!()
}
