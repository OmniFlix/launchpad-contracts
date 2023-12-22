use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Coin, Timestamp};

#[cw_serde]
#[derive(QueryResponses)]
pub enum RoundWhitelistQueryMsgs {
    #[returns(Vec<Round>)]
    Rounds {},
    #[returns(Round)]
    Round { round_index: u32 },
    #[returns(IsActiveResponse)]
    IsActive {},
    #[returns(Round)]
    ActiveRound {},
    #[returns(MembersResponse)]
    Members {
        round_index: u32,
        start_after: Option<String>,
        limit: Option<u32>,
    },
    // Returns price of the active round
    #[returns(MintPriceResponse)]
    Price {},
    #[returns(IsMemberResponse)]
    IsMember { address: String },
}
#[cw_serde]
pub struct MembersResponse {
    pub members: Vec<String>,
}

#[cw_serde]
pub struct IsMemberResponse {
    pub is_member: bool,
}

#[cw_serde]
pub struct IsActiveResponse {
    pub is_active: bool,
}

#[cw_serde]
pub struct MintPriceResponse {
    pub mint_price: Coin,
}

#[cw_serde]
pub enum Round {
    WhitelistAddresses {
        addresses: Vec<Addr>,
        start_time: Timestamp,
        end_time: Timestamp,
        mint_price: Coin,
        round_per_address_limit: u32,
    },
    WhitelistCollection {
        collection_id: String,
        start_time: Timestamp,
        end_time: Timestamp,
        mint_price: Coin,
        round_per_address_limit: u32,
    },
}
