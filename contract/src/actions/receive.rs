#[cfg(not(feature = "library"))]
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response, Uint128};

use crate::error::ContractError;

pub fn deposit(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _sender: String,
    _amount: Uint128,
) -> Result<Response, ContractError> {
    Ok(Response::new().add_attributes(vec![("action", "deposit")]))
}

pub fn swap(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _sender: String,
    _amount: Uint128,
) -> Result<Response, ContractError> {
    Ok(Response::new().add_attributes(vec![("action", "swap")]))
}
