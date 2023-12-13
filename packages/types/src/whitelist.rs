use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Coin, Timestamp};

#[cw_serde]
#[derive(QueryResponses)]
pub enum WhitelistQueryMsgs {
    #[returns(HasStartedResponse)]
    HasStarted {},
    #[returns(HasEndedResponse)]
    HasEnded {},
    #[returns(IsActiveResponse)]
    IsActive {},
    #[returns(MembersResponse)]
    Members {
        start_after: Option<String>,
        limit: Option<u32>,
    },
    #[returns(HasMemberResponse)]
    HasMember { member: String },
    #[returns(Config)]
    Config {},
    #[returns(PerAddressLimitResponse)]
    PerAddressLimit {},
}

#[cw_serde]
pub struct Config {
    pub admin: Addr,
    pub start_time: Timestamp,
    pub end_time: Timestamp,
    pub mint_price: Coin,
    pub per_address_limit: u32,
    pub member_limit: u32,
    pub is_frozen: bool,
}

#[cw_serde]
pub struct MembersResponse {
    pub members: Vec<String>,
}

#[cw_serde]
pub struct HasMemberResponse {
    pub has_member: bool,
}

#[cw_serde]
pub struct HasStartedResponse {
    pub has_started: bool,
}

#[cw_serde]
pub struct HasEndedResponse {
    pub has_ended: bool,
}

#[cw_serde]
pub struct IsActiveResponse {
    pub is_active: bool,
}

#[cw_serde]
pub struct PerAddressLimitResponse {
    pub per_address_limit: u32,
}
