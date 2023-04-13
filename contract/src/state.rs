use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Decimal, Timestamp, Uint128};
use cw_storage_plus::{Item, Map};

pub const CONFIG: Item<Config> = Item::new("config");

#[cw_serde]
pub struct Config {
    pub admin: Addr,
    pub swap_fee_rate: Decimal,
}

// key - token_addr: &Addr
pub const TOKENS: Map<&Addr, Token> = Map::new("tokens");

#[cw_serde]
pub struct Token {
    pub symbol: String, // TODO: check if it's needed
    pub price_feed_id_str: String,
    pub bonded: Vec<Uint128>,    // providing liquidity +, fee-sharing +
    pub unbonded: Vec<Uint128>,  // providing liquidity -, fee-sharing - | ready for withdrawing
    pub requested: Vec<Uint128>, // providing liquidity +, fee-sharing - | will become unbonded when time >= counter
    pub swapped_in: Vec<Uint128>,
    pub swapped_out: Vec<Uint128>,
}

impl Token {
    pub fn new(symbol: &str, price_feed_id_str: &str) -> Self {
        Token {
            symbol: symbol.to_string(),
            price_feed_id_str: price_feed_id_str.to_string(),
            bonded: vec![],
            unbonded: vec![],
            requested: vec![],
            swapped_in: vec![],
            swapped_out: vec![],
        }
    }
}

// key - address: &Addr
pub const PROVIDERS: Map<&Addr, Vec<Asset>> = Map::new("providers");

#[cw_serde]
pub struct Asset {
    pub token_addr: String,
    pub bonded: Uint128,    // providing liquidity +, fee-sharing +
    pub unbonded: Uint128,  // providing liquidity -, fee-sharing - | ready for withdrawing
    pub requested: Uint128, // providing liquidity +, fee-sharing - | will become unbonded when time >= counter
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
