use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Coin, Timestamp, Uint128};

use crate::state::Round;

#[cw_serde]
pub struct InstantiateMsg {
    pub admin: Option<String>,
    pub rounds: Vec<Round>,
}

#[cw_serde]
pub enum ExecuteMsg {
    RemoveRound { round_index: u32 },
    AddRound { round: Round },
    PrivateMint { minter: String, admin: String },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum RoundWhitelistQueryMsgs {
    #[returns(Vec<Round>)]
    Rounds {},
    #[returns(Round)]
    Round { round_index: u32 },
    #[returns(bool)]
    IsActive { round_index: u32 },
    #[returns(Round)]
    ActiveRound {},
    #[returns(Vec<String>)]
    Members {
        round_index: u32,
        start_after: Option<String>,
        limit: Option<u32>,
    },
    // Returns price of the active round
    #[returns(Coin)]
    Price {},
    #[returns(bool)]
    IsMember { address: String },
}
