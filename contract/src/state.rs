use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cw_storage_plus::Item;

pub const PYTH: Item<Pyth> = Item::new("pyth");

#[cw_serde]
pub struct Pyth {
    pub pyth_contract_addr: Addr,
}
