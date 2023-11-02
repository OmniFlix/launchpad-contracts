use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Timestamp, Uint128};

use crate::state::Round;

#[cw_serde]
pub struct InstantiateMsg {
    pub collection_details: CollectionDetails,
    // Sender will be factory so we expect creator to be sent from the factory
    pub creator: Option<String>,
    pub mint_price: Uint128,
    // Factory sould check denom against the params if denoms is valid and whitelisted
    pub mint_denom: String,
    // Minting start time
    pub start_time: Timestamp,
    pub per_address_limit: u32,
    pub rounds: Option<Vec<Round>>,
    // We expect user to send a string between 0 and 1
    // FE "0.1"
    pub royalty_ratio: String,
    pub payment_collector: Option<String>,
}

#[cw_serde]
pub struct CollectionDetails {
    pub name: String,
    pub description: String,
    pub preview_uri: String,
    pub schema: String,
    pub symbol: String,
    pub id: String,
    pub extensible: bool,
    pub nsfw: bool,
    pub num_tokens: u32,
    pub base_uri: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    Mint {},
    MintAdmin {
        recipient: String,
        denom_id: Option<String>,
    },
    RemoveRound {
        round_index: u32,
    },
    AddRound {
        round: Round,
    },
    UpdateCollectiomRound {
        round_index: u32,
        round: Round,
    },
    UpdateWhitelistRound {
        start_time: Option<Timestamp>,
        end_time: Option<Timestamp>,
        mint_price: Option<Uint128>,
        round_limit: Option<u32>,
    },
    BurnRemainingTokens {},
    UpdateRoyaltyRatio {
        ratio: String,
    },
    UpdateMintPrice {
        mint_price: Uint128,
    },
    RandomizeList {},
}

#[cw_serde]
pub enum QueryMsg {
    Config {},
    Collection {},
    MintedTokens { address: String },
    MintableTokens {},
    TotalTokens {},
    Whitelist {},
}

#[cw_serde]
pub enum WhitelistQueryMsg {
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
