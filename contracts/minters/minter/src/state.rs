use std::u32;

use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

use minter_types::{CollectionDetails, Config, Token, TokenDetails, UserDetails};

pub const CONFIG: Item<Config> = Item::new("config");
pub const COLLECTION: Item<CollectionDetails> = Item::new("collection");
pub const TOKEN_DETAILS: Item<TokenDetails> = Item::new("token_details");
// Index of mintable tokens and denom ids
pub const MINTABLE_TOKENS: Map<u32, Token> = Map::new("mintable_tokens");
// Total number of tokens
pub const TOTAL_TOKENS_REMAINING: Item<u32> = Item::new("total_tokens_remaining");
// Address and number of tokens minted
pub const MINTED_TOKENS: Map<Addr, UserDetails> = Map::new("minted_tokens");
