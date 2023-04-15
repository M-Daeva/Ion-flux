use cosmwasm_std::Uint128;
#[cfg(not(feature = "library"))]
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};
use cw2::set_contract_version;

use crate::{
    actions::math::str_to_dec,
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

pub fn init(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    CONFIG.save(
        deps.storage,
        &Config {
            admin: info.sender,
            swap_fee_rate: str_to_dec(SWAP_FEE_RATE),
            window: Uint128::from(WINDOW),
            unbonding_period: Uint128::from(UNBONDING_PERIOD),
        },
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
