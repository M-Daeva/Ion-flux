use cosmwasm_schema::{cw_serde, QueryResponses};

// preventing optimizer warning message
#[allow(unused_imports)]
use pyth_sdk_cw::PriceFeedResponse;

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(PriceFeedResponse)]
    QueryPrice { price_feed_id_str: String },
}
