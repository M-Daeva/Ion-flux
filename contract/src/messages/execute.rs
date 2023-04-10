use cosmwasm_schema::cw_serde;

use cw20::Cw20ReceiveMsg;

use cosmwasm_std::Uint128;

#[cw_serde]
pub enum ExecuteMsg {
    Receive(Cw20ReceiveMsg),
    Withdraw { token: String, amount: Uint128 },
}
