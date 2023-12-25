use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Coin, Timestamp, Uint128};

#[cw_serde]
pub struct InstantiateMsg {
    pub collection_details: CollectionDetails,
    pub admin: Option<String>,
    pub mint_price: Uint128,
    // Factory sould check denom against the params if denoms is valid and whitelisted.
    pub mint_denom: String,
    // Public minting start time
    pub start_time: Timestamp,
    pub per_address_limit: u32,
    // We expect user to send a string between 0 and 1
    // FE "0.1"
    pub royalty_ratio: String,
    pub payment_collector: Option<String>,
    // Whitelist address if any
    pub whitelist_address: Option<String>,
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
    pub uri: String,
    pub uri_hash: String,
    pub data: String,
}
