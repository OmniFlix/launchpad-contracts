use cosmwasm_schema::cw_serde;
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
}
