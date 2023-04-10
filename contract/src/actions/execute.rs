#[cfg(not(feature = "library"))]
use cosmwasm_std::{to_binary, DepsMut, Env, MessageInfo, Response, Uint128, WasmMsg};

use cw20::Cw20ExecuteMsg;

use crate::error::ContractError;

pub fn withdraw(
    _deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    token: String,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let cw_send_msg = Cw20ExecuteMsg::Transfer {
        recipient: info.sender.to_string(),
        amount,
    };

    let msg = WasmMsg::Execute {
        contract_addr: token,
        msg: to_binary(&cw_send_msg)?,
        funds: vec![],
    };

    Ok(Response::new()
        .add_message(msg)
        .add_attributes(vec![("action", "withdraw")]))
}
