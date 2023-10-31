use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Coin, Timestamp};

#[cw_serde]
pub enum WhitelistQueryMsgs {
    HasStarted {},
    HasEnded {},
    IsActive {},
    Members {
        start_after: Option<String>,
        limit: Option<u32>,
    },
    HasMember {
        member: String,
    },
    Config {},
    PerAddressLimit {},
}
