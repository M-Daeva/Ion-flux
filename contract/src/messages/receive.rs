use cosmwasm_schema::cw_serde;

#[cw_serde]
pub enum ReceiveMsg {
    Deposit {},
    Swap { token_out_addr: String },
    SwapMocked { token_out_addr: String },
}
