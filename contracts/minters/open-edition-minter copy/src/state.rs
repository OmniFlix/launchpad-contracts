use std::{ops::Add, u32};

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, StdError, Storage};
use cw_storage_plus::{Item, Map};

use minter_types::{CollectionDetails, Config, UserDetails};
pub const MINTED_COUNT: Map<EditionNumber, u32> = Map::new("minted_count");
pub const CURRENT_EDITION: Item<u32> = Item::new("current_edition");
pub const MINTED_TOKENS_KEY: &str = "minted_tokens";
pub const LAST_MINTED_TOKEN_ID: Item<u32> = Item::new("last_minted_token_id");
pub type EditionNumber = u32;
#[cw_serde]
pub struct EditionParams {
    pub config: Config,
    pub collection: CollectionDetails,
}
pub const EDITIONS: Map<EditionNumber, EditionParams> = Map::new("editions");

pub fn last_token_id(store: &mut dyn Storage, edition_number: u32) -> u32 {
    let minted_count = MINTED_COUNT.load(store, edition_number).unwrap_or_default();
    minted_count
}

pub struct MintedTokens<'a>(Map<'a, (EditionNumber, Addr), UserDetails>);

impl<'a> MintedTokens<'a> {
    pub const fn new(storage_key: &'a str) -> Self {
        MintedTokens(Map::new(storage_key))
    }

    pub fn load(
        &self,
        store: &dyn Storage,
        edition: EditionNumber,
        address: Addr,
    ) -> Result<UserDetails, StdError> {
        let user_details = self.0.load(store, (edition, address))?;
        Ok(user_details)
    }

    pub fn save(
        &self,
        store: &mut dyn Storage,
        edition: EditionNumber,
        address: Addr,
        data: &UserDetails,
    ) {
        self.0.save(store, (edition, address), data).unwrap();
    }
}
