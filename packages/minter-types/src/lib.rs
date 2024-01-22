use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Coin, Decimal, Timestamp, Uint128};

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
    pub base_uri: String,
    pub uri: String,
    pub uri_hash: String,
    pub data: String,
}

#[cw_serde]
pub struct MinterInstantiateMsg<T> {
    pub collection_details: CollectionDetails,
    pub init: T,
}

#[cw_serde]
pub struct UserDetails {
    pub minted_tokens: Vec<Token>,
    pub total_minted_count: u32,
}

impl Default for UserDetails {
    fn default() -> Self {
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
    pub token_limit: Option<u32>,
}
