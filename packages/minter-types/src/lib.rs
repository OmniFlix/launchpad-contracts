use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Coin, Decimal, Timestamp, Uint128};

#[cw_serde]
pub struct InstantiateMsg {
    pub collection_details: CollectionDetails,
    pub admin: Option<String>,
    pub mint_price: Uint128,
    // Factory sould check denom against the params if denoms is valid and whitelisted.
    pub mint_denom: String,
    // Public minting start time
    pub start_time: Timestamp,
    pub end_time: Option<Timestamp>,
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

#[cw_serde]
pub struct UserDetails {
    pub minted_tokens: Vec<Token>,
    pub total_minted_count: u32,
}

impl UserDetails {
    pub fn new() -> Self {
        UserDetails {
            minted_tokens: Vec::new(),
            total_minted_count: 0,
        }
    }
}

#[cw_serde]
pub struct Token {
    pub token_id: String,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(CollectionDetails)]
    Collection {},
    #[returns(Config)]
    Config {},
    #[returns(Vec<Token>)]
    MintableTokens {},
    #[returns(UserDetails)]
    MintedTokens { address: String },
    #[returns(u32)]
    TotalTokens {},
}

#[cw_serde]
pub struct Config {
    pub per_address_limit: u32,
    pub payment_collector: Addr,
    pub start_time: Timestamp,
    pub end_time: Option<Timestamp>,
    pub mint_price: Coin,
    pub royalty_ratio: Decimal,
    pub admin: Addr,
    pub whitelist_address: Option<Addr>,
}
