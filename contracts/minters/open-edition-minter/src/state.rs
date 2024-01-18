use std::u32;

use cosmwasm_std::{Addr, Storage};
use cw_storage_plus::{Item, Map};

use open_edition_minter_types::{CollectionDetails, Config, UserDetails};

pub const CONFIG: Item<Config> = Item::new("config");
pub const COLLECTION: Item<CollectionDetails> = Item::new("collection");
pub const MINTED_COUNT: Item<u32> = Item::new("minted_count");
// Address and number of tokens minted
pub const MINTED_TOKENS: Map<Addr, UserDetails> = Map::new("minted_tokens");

pub fn last_token_id(store: &mut dyn Storage) -> u32 {
    let minted_count = MINTED_COUNT.load(store).unwrap_or_default();
    minted_count
}
