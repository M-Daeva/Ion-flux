use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Decimal, Timestamp, Uint128};
use cw_storage_plus::{Item, Map};

pub const CONFIG: Item<Config> = Item::new("config");

#[cw_serde]
pub struct Config {
    pub admin: Addr,
    pub swap_fee_rate: Decimal,
}

// key - symbol: &str
pub const TOKENS: Map<&str, Token> = Map::new("tokens");

#[cw_serde]
pub struct Token {
    pub token_addr: Addr,
    pub price_feed_id_str: String,
    // pub weight: Decimal,
    // pub price: Decimal,
}

// key - address: &Addr
pub const PROVIDERS: Map<&Addr, Vec<Asset>> = Map::new("providers");

#[cw_serde]
pub struct Asset {
    pub symbol: String,
    pub bonded: Uint128,    // used in providing liquidity
    pub unbonded: Uint128,  // ready for withdrawing
    pub requested: Uint128, // will become unbonded when time >= counter
    pub counter: Timestamp,
    pub rewards: Uint128,
}

pub const PYTH: Item<Pyth> = Item::new("pyth");

#[cw_serde]
pub struct Pyth {
    pub pyth_contract_addr: Addr,
}

#[cw_serde]
pub struct Sample {
    pub value: Uint128,
    pub timestamp: Uint128,
}

// TODO: use Timestamp
impl Sample {
    pub fn new(value: Uint128, timestamp: Uint128) -> Self {
        Sample { value, timestamp }
    }
}
