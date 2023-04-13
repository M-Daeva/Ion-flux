use cosmwasm_schema::cw_serde;

use cw20::Cw20ReceiveMsg;

use cosmwasm_std::{Decimal, Uint128};

#[cw_serde]
pub enum ExecuteMsg {
    Receive(Cw20ReceiveMsg),
    UpdateConfig {
        admin: Option<String>,
        swap_fee_rate: Option<Decimal>,
    },
    UpdateToken {
        token_addr: String,
        symbol: String,
        price_feed_id_str: String,
    },
    Unbond {
        token_addr: String,
        amount: Uint128,
    },
    Withdraw {
        token_addr: String,
        amount: Uint128,
    },
    Claim {},
    SwapAndClaim {
        token_addr: String,
    },
}
