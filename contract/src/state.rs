use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Decimal, Timestamp, Uint128};
use cw_storage_plus::{Item, Map};

pub const CONFIG: Item<Config> = Item::new("config");

#[cw_serde]
pub struct Config {
    pub admin: Addr,
    pub swap_fee_rate: Decimal,
    pub window: Uint128,
    pub unbonding_period: Uint128,
}

// key - token_addr: &Addr
pub const TOKENS: Map<&Addr, Token> = Map::new("tokens");

#[cw_serde]
pub struct Token {
    pub symbol: String, // TODO: check if it's needed
    pub price_feed_id_str: String,
    pub bonded: (Vec<Sample>, Uint128), // providing liquidity +, fee-sharing +
    pub unbonded: (Vec<Sample>, Uint128), // providing liquidity -, fee-sharing - | ready for withdrawing
    pub requested: (Vec<Sample>, Uint128), // providing liquidity +, fee-sharing - | will become unbonded when time >= counter
    pub swapped_in: (Vec<Sample>, Uint128),
    pub swapped_out: (Vec<Sample>, Uint128),
}

impl Token {
    pub fn new(symbol: &str, price_feed_id_str: &str) -> Self {
        let zero = Uint128::zero();

        Token {
            symbol: symbol.to_string(),
            price_feed_id_str: price_feed_id_str.to_string(),
            bonded: (vec![], zero),
            unbonded: (vec![], zero),
            requested: (vec![], zero),
            swapped_in: (vec![], zero),
            swapped_out: (vec![], zero),
        }
    }
}

// key - address: &Addr
pub const PROVIDERS: Map<&Addr, Vec<Asset>> = Map::new("providers");

#[cw_serde]
pub struct Asset {
    pub token_addr: Addr,
    pub bonded: Uint128,    // providing liquidity +, fee-sharing +
    pub unbonded: Uint128,  // providing liquidity -, fee-sharing - | ready for withdrawing
    pub requested: Uint128, // providing liquidity +, fee-sharing - | will become unbonded when time >= counter
    pub counter: Timestamp,
    pub rewards: Uint128,
}

impl Asset {
    pub fn new(token_addr: &Addr, timestamp: &Timestamp) -> Self {
        let zero = Uint128::zero();

        Asset {
            token_addr: token_addr.to_owned(),
            bonded: zero,
            unbonded: zero,
            requested: zero,
            counter: timestamp.to_owned(),
            rewards: zero,
        }
    }
}

pub const PYTH: Item<Pyth> = Item::new("pyth");

#[cw_serde]
pub struct Pyth {
    pub pyth_contract_addr: Addr,
}

#[cw_serde]
pub struct Sample {
    pub value: Uint128,
    pub timestamp: Timestamp,
}

impl Sample {
    pub fn new(value: Uint128, timestamp: Timestamp) -> Self {
        Sample { value, timestamp }
    }
}
