use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Coin, Timestamp, Uint128};

use whitelist_types::Round;

#[cw_serde]
pub enum ExecuteMsg {
    RemoveRound { round_index: u32 },
    AddRound { round: Round },
    PrivateMint { minter: String, admin: String },
}

#[cw_serde]
pub struct ConfigResponse {
    pub num_members: u32,
    pub per_address_limit: u32,
    pub member_limit: u32,
    pub start_time: Timestamp,
    pub end_time: Timestamp,
    pub mint_price: Coin,
    pub is_active: bool,
}
