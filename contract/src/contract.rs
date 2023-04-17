#[cfg(not(feature = "library"))]
use cosmwasm_std::{
    entry_point, from_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};

use cw20::Cw20ReceiveMsg;

use crate::{
    actions::{
        execute::{claim, swap_and_claim, unbond, update_config, update_token, withdraw},
        instantiate::init,
        migrate::migrate_contract,
        query::{query_balances, query_price, query_provider, query_tokens},
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
        ExecuteMsg::UpdateConfig {
            admin,
            swap_fee_rate,
            window,
            unbonding_period,
        } => update_config(
            deps,
            env,
            info,
            admin,
            swap_fee_rate,
            window,
            unbonding_period,
        ),
        ExecuteMsg::UpdateToken {
            token_addr,
            symbol,
            price_feed_id_str,
        } => update_token(deps, env, info, token_addr, symbol, price_feed_id_str),
        ExecuteMsg::Unbond { token_addr, amount } => unbond(deps, env, info, token_addr, amount),
        ExecuteMsg::Withdraw { token_addr, amount } => {
            withdraw(deps, env, info, token_addr, amount)
        }
        // TODO
        ExecuteMsg::Claim {} => claim(deps, env, info),
        // TODO
        ExecuteMsg::SwapAndClaim { token_addr } => swap_and_claim(deps, env, info, token_addr),
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
        // TODO
        ReceiveMsg::Swap { token_addr_out } => {
            swap(deps, env, info, sender, amount, token_addr_out)
        }
    }
}

/// Exposes all the queries available in the contract
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::QueryProvider { address } => query_provider(deps, env, address),
        // TODO
        QueryMsg::QueryTokens {} => query_tokens(deps, env),
        // TODO
        QueryMsg::QueryBalances {} => query_balances(deps, env),
        QueryMsg::QueryPrice { price_feed_id_str } => query_price(deps, env, price_feed_id_str),
    }
}

/// Used for contract migration
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, env: Env, msg: MigrateMsg) -> Result<Response, ContractError> {
    migrate_contract(deps, env, msg)
}
