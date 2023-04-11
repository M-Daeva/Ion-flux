use cosmwasm_schema::cw_serde;

use cosmwasm_std::Uint128;

#[cw_serde]
pub struct Balance {
    pub symbol: String,
    pub amount: Uint128,
}
