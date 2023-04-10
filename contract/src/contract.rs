#[cfg(not(feature = "library"))]
use cosmwasm_std::{
    entry_point, from_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};

use cw20::Cw20ReceiveMsg;

use crate::{
    actions::{
        execute::withdraw,
        instantiate::init,
        migrate::migrate_contract,
        query::query_price,
        receive::{deposit, swap},
    },
    error::ContractError,
    messages::{
        execute::ExecuteMsg, instantiate::InstantiateMsg, migrate::MigrateMsg, query::QueryMsg,
        receive::ReceiveMsg,
    },
};

/// Creates a new contract with the specified parameters packed in the "msg" variable
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    init(deps, env, info, msg)
}

/// Exposes all the execute functions available in the contract
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Receive(msg) => receive(deps, env, info, msg),
        ExecuteMsg::Withdraw { token, amount } => withdraw(deps, env, info, token, amount),
    }
}

/// Exposes all the receive functions available in the contract
pub fn receive(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    wrapper: Cw20ReceiveMsg,
) -> Result<Response, ContractError> {
    let Cw20ReceiveMsg {
        sender,
        amount,
        msg,
    } = wrapper;

    match from_binary(&msg)? {
        ReceiveMsg::Deposit {} => deposit(deps, env, info, sender, amount),
        ReceiveMsg::Swap {} => swap(deps, env, info, sender, amount),
    }
}

/// Exposes all the queries available in the contract
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::QueryPrice { price_feed_id_str } => query_price(deps, env, price_feed_id_str),
    }
}

/// Used for contract migration
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, env: Env, msg: MigrateMsg) -> Result<Response, ContractError> {
    migrate_contract(deps, env, msg)
}
