use std::u32;

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Decimal, Timestamp, Uint128};
use cw_storage_plus::{Item, Map};

use crate::msg::CollectionDetails;

#[cw_serde]
pub struct Config {
    pub per_address_limit: u32,
    pub payment_collector: Addr,
    pub whitelist_address: Option<Addr>,
    pub mint_denom: String,
    pub start_time: Timestamp,
    pub mint_price: Uint128,
    pub royalty_ratio: Decimal,
    pub creator: Addr,
}

#[cw_serde]
pub struct Token {
    pub token_id: String,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const COLLECTION: Item<CollectionDetails> = Item::new("collection");
// Index of mintable tokens and denom ids
pub const MINTABLE_TOKENS: Map<u32, Token> = Map::new("mintable_tokens");
// Total number of tokens
pub const TOTAL_TOKENS_REMAINING: Item<u32> = Item::new("total_tokens_remaining");
// Address and number of tokens minted
pub const MINTED_TOKENS: Map<Addr, Vec<Token>> = Map::new("minted_tokens");
