use std::u32;

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Coin, Decimal, Timestamp};
use cw_storage_plus::{Item, Map};

use crate::msg::CollectionDetails;

#[cw_serde]
pub struct Config {
    pub per_address_limit: u32,
    pub payment_collector: Addr,
    pub start_time: Timestamp,
    pub mint_price: Coin,
    pub royalty_ratio: Decimal,
    pub admin: Addr,
}

#[cw_serde]
pub struct Token {
    pub token_id: String,
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

pub const CONFIG: Item<Config> = Item::new("config");
pub const COLLECTION: Item<CollectionDetails> = Item::new("collection");
// Index of mintable tokens and denom ids
pub const MINTABLE_TOKENS: Map<u32, Token> = Map::new("mintable_tokens");
// Total number of tokens
pub const TOTAL_TOKENS_REMAINING: Item<u32> = Item::new("total_tokens_remaining");
// Address and number of tokens minted
pub const MINTED_TOKENS: Map<Addr, UserDetails> = Map::new("minted_tokens");
