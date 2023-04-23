use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Decimal, Timestamp, Uint128};
use cw_storage_plus::{Item, Map};

use crate::actions::math::str_to_dec;

pub const CHAIN_ID_MOCKED: &str = "cw_multi_test";

pub const CONFIG: Item<Config> = Item::new("config");

#[cw_serde]
pub struct Config {
    pub admin: Addr,
    pub swap_fee_rate: Decimal,
    pub window: Uint128,
    pub unbonding_period: Uint128,
    pub price_age: Uint128,
    chain_id_mocked: String,
}

impl Config {
    pub fn new(
        admin: &Addr,
        swap_fee_rate: &str,
        window: u128,
        unbonding_period: u128,
        price_age: u128,
    ) -> Self {
        Config {
            admin: admin.to_owned(),
            swap_fee_rate: str_to_dec(swap_fee_rate),
            window: Uint128::from(window),
            unbonding_period: Uint128::from(unbonding_period),
            price_age: Uint128::from(price_age),
            chain_id_mocked: String::from(CHAIN_ID_MOCKED),
        }
    }

    pub fn get_chain_id(&self) -> String {
        self.chain_id_mocked.clone()
    }
}

// key - token_addr: &Addr
pub const TOKENS: Map<&Addr, Token> = Map::new("tokens");

// time series/sma values reflecting overall liquidity movement
#[cw_serde]
pub struct Token {
    pub symbol: String,
    pub price_feed_id_str: String,
    pub bonded: (Vec<Sample>, Uint128),
    pub requested: (Vec<Sample>, Uint128),
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
            requested: (vec![], zero),
            swapped_in: (vec![], zero),
            swapped_out: (vec![], zero),
        }
    }
}

// key - address: &Addr
pub const PROVIDERS: Map<&Addr, Vec<Asset>> = Map::new("providers");

// cumulative values reflecting providers balances state
#[cw_serde]
pub struct Asset {
    pub token_addr: Addr,
    pub bonded: Uint128,    // providing liquidity +, fee-sharing +
    pub unbonded: Uint128,  // providing liquidity -, fee-sharing - | ready for withdrawing
    pub requested: Uint128, // providing liquidity +, fee-sharing - | will be unbonded when time >= counter
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
