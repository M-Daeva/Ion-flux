use cosmwasm_schema::{cw_serde, QueryResponses};

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
    #[returns(Vec<Asset>)]
    QueryProvider { address: String },
    #[returns(Vec<Token>)]
    QueryTokens {},
    #[returns(Vec<Balance>)]
    QueryBalances {},
    #[returns(PriceFeedResponse)]
    QueryPrice { price_feed_id_str: String },
}
