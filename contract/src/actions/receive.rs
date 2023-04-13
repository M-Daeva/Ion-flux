#[cfg(not(feature = "library"))]
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response, Uint128};

use crate::{
    error::ContractError,
    state::{Asset, PROVIDERS, TOKENS},
};

pub fn deposit(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    sender: String,
    amount: Uint128,
) -> Result<Response, ContractError> {
    // check if token is supported
    TOKENS.load(deps.storage, &info.sender)?;

    let provider_addr = deps.api.addr_validate(&sender)?;

    // check if provider exists or create new one
    let provider = match PROVIDERS.load(deps.storage, &provider_addr) {
        Ok(x) => x,
        _ => vec![],
    };

    let provider_updated: Vec<Asset> = provider
        .iter()
        .map(|x| {
            // check unbonding counter
            let (requested, unbonded) = if !x.requested.is_zero() && (x.counter <= env.block.time) {
                (Uint128::zero(), x.requested)
            } else {
                (x.requested, x.unbonded)
            };

            let bonded = if x.token_addr == info.sender {
                x.bonded + amount
            } else {
                x.bonded
            };

            // TODO: update TOKENS

            Asset {
                bonded,
                unbonded,
                requested,
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
