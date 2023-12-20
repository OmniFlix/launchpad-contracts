use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Coin, Timestamp, Uint128};

#[cw_serde]
pub struct InstantiateMsg {
    pub admin: Option<String>,
    pub start_time: Timestamp,
    pub end_time: Timestamp,
    pub mint_price: Coin,
    pub per_address_limit: u32,
    pub members: Vec<String>,
    pub member_limit: u32,
}

#[cw_serde]
pub enum ExecuteMsg {
    UpdateStartTime { start_time: Timestamp },
    UpdateEndTime { end_time: Timestamp },
    AddMembers { addresses: Vec<String> },
    RemoveMembers { addresses: Vec<String> },
    UpdateMintPrice { mint_price: Coin },
    UpdatePerAddressLimit { amount: u32 },
    IncreaseMemberLimit { amount: u32 },
    UpdateAdmin { admin: String },
    Freeze {},
}
