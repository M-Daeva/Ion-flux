use cosmwasm_schema::{cw_serde, QueryResponses};

#[allow(unused_imports)] // preventing optimizer warning message
use cosmwasm_std::Addr;

#[allow(unused_imports)] // preventing optimizer warning message
use crate::{
    messages::response::Balance,
    state::{Asset, Token},
};

#[allow(unused_imports)] // preventing optimizer warning message
use pyth_sdk_cw::PriceFeedResponse;

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Vec<(Addr, Vec<Asset>)>)]
    QueryProviders { address: Option<String> },
    #[returns(Vec<(Addr, Token)>)]
    QueryTokens { address: Option<String> },
    #[returns(Vec<Balance>)]
    QueryBalances { address: Option<String> },
    #[returns(PriceFeedResponse)]
    QueryPrice { price_feed_id_str: String },
}
