use cosmwasm_schema::cw_serde;

#[cw_serde]
pub enum ReceiveMsg {
    Deposit {},
    Swap { token_addr_out: String },
}
