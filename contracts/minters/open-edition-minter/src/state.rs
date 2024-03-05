use std::u32;

use cosmwasm_std::{Addr, Storage};
use cw_storage_plus::{Item, Map};

use minter_types::{AuthDetails, CollectionDetails, Config, TokenDetails, UserDetails};

pub const CONFIG: Item<Config> = Item::new("config");
pub const COLLECTION: Item<CollectionDetails> = Item::new("collection");
pub const TOKEN_DETAILS: Item<TokenDetails> = Item::new("token_details");
pub const MINTED_COUNT: Item<u32> = Item::new("minted_count");
// Address and number of tokens minted
pub const USER_MINTING_DETAILS: Map<Addr, UserDetails> = Map::new("user_minting_details");
pub const AUTH_DETAILS: Item<AuthDetails> = Item::new("auth_details");

pub fn last_token_id(store: &mut dyn Storage) -> u32 {
    MINTED_COUNT.load(store).unwrap_or_default()
}
