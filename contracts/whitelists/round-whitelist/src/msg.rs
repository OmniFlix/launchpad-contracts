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
    UpdateRound { round_index: u32, round: Round },
    RemoveRound { round_index: u32 },
    AddRound { round: Round },
    PrivatelyMint { minter: String },
}

#[cw_serde]
pub enum WhitelistQueryMsg {
    Rounds {},
    Round {
        round_index: u32,
    },
    HasStarted {
        round_index: u32,
    },
    HasEnded {
        round_index: u32,
    },
    IsActive {
        round_index: u32,
    },
    ActiveRound {},

    Members {
        round_index: u32,
        start_after: Option<String>,
        limit: Option<u32>,
    },
    // This query is for minter contract to check mint price for the active round
    // Will be implemented for the classical whitelist contract as well

    // Returns mint price with Coin type
    Price {},
}
