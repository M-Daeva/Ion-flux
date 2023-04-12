use cosmwasm_schema::cw_serde;

use cosmwasm_std::{Addr, Uint128};

#[cw_serde]
pub struct Balance {
    pub token_addr: Addr,
    pub amount: Uint128,
}
