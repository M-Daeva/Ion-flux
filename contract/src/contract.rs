#[cfg(not(feature = "library"))]
use cosmwasm_std::{
    entry_point, from_binary, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response,
    StdResult,
};

use cw20::Cw20ReceiveMsg;

use crate::{
    actions::{
        execute::{claim, swap_and_claim, unbond, update_config, update_token, withdraw},
        instantiate::init,
        migrate::migrate_contract,
        query::{query_aprs, query_balances, query_prices, query_providers, query_tokens},
        receive::{deposit, swap, swap_mocked},
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
            price_age,
        } => update_config(
            deps,
            env,
            info,
            admin,
            swap_fee_rate,
            window,
            unbonding_period,
            price_age,
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
        ReceiveMsg::Swap { token_out_addr } => {
            swap(deps, env, info, sender, amount, token_out_addr)
        }
        ReceiveMsg::SwapMocked { token_out_addr } => {
            swap_mocked(deps, env, info, sender, amount, token_out_addr)
        }
    }
}

/// Exposes all the queries available in the contract
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::QueryAprs { address_list } => to_binary(&query_aprs(deps, env, address_list)?),
        QueryMsg::QueryProviders { address_list } => {
            to_binary(&query_providers(deps, env, address_list)?)
        }
        QueryMsg::QueryTokens { address_list } => {
            to_binary(&query_tokens(deps, env, address_list)?)
        }
        QueryMsg::QueryBalances { address_list } => {
            to_binary(&query_balances(deps, env, address_list)?)
        }
        QueryMsg::QueryPrices { address_list } => {
            to_binary(&query_prices(deps, env, address_list)?)
        }
    }
}

/// Used for contract migration
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, env: Env, msg: MigrateMsg) -> Result<Response, ContractError> {
    migrate_contract(deps, env, msg)
}
