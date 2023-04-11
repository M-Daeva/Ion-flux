use cosmwasm_schema::cw_serde;

use cw20::Cw20ReceiveMsg;

use cosmwasm_std::{Decimal, Uint128};

#[cw_serde]
pub enum ExecuteMsg {
    Receive(Cw20ReceiveMsg),
    UpdateConfig {
        admin: Option<String>,
        swap_fee: Option<Decimal>,
    },
    UpdateTokens {
        symbol: String,
        token_addr: String,
        price_feed_id_str: String,
    },
    Unbond {
        symbol: String,
        amount: Uint128,
    },
    Withdraw {
        symbol: String,
        amount: Uint128,
    },
    Claim {},
    SwapAndClaim {
        symbol: String,
    },
}
