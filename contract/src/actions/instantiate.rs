#[cfg(not(feature = "library"))]
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};
use cw2::set_contract_version;

use crate::{
    error::ContractError,
    messages::instantiate::InstantiateMsg,
    state::{Config, Pyth, CONFIG, PYTH},
};

const CONTRACT_NAME: &str = "crates.io:ion-flux";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const PYTH_CONTRACT_ADDR: &str = "inj1z60tg0tekdzcasenhuuwq3htjcd5slmgf7gpez";

const SWAP_FEE_RATE: &str = "0.003";
const WINDOW: u128 = 30 * 60 * 1_000_000_000;
const UNBONDING_PERIOD: u128 = 60 * 60 * 1_000_000_000;
const PRICE_AGE: u128 = 4_000_000;

pub fn init(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    CONFIG.save(
        deps.storage,
        &Config::new(
            &info.sender,
            SWAP_FEE_RATE,
            WINDOW,
            UNBONDING_PERIOD,
            PRICE_AGE,
        ),
    )?;

    PYTH.save(
        deps.storage,
        &Pyth {
            pyth_contract_addr: deps.api.addr_validate(PYTH_CONTRACT_ADDR)?,
        },
    )?;

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new().add_attributes(vec![("action", "instantiate")]))
}
