use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, StdError, Storage};
use cw_storage_plus::{Item, Map};
use std::u32;

use minter_types::{AuthDetails, CollectionDetails, Config, TokenDetails, UserDetails};

pub const DROP_MINTED_COUNT: Map<DropID, u32> = Map::new("drop_minted_count");
pub const CURRENT_DROP_ID: Item<u32> = Item::new("current_edition");
pub const LAST_MINTED_TOKEN_ID: Item<u32> = Item::new("last_minted_token_id");
pub const AUTH_DETAILS: Item<AuthDetails> = Item::new("auth_details");

pub type DropID = u32;
#[cw_serde]
pub struct DropParams {
    pub config: Config,
    pub token_details: TokenDetails,
}
pub const COLLECTION: Item<CollectionDetails> = Item::new("collection");
pub const DROPS: Map<DropID, DropParams> = Map::new("editions");

pub const USER_MINTED_TOKENS_KEY: &str = "user_minted_tokens";
pub struct UserMintedTokens<'a>(Map<'a, (DropID, Addr), UserDetails>);

impl<'a> UserMintedTokens<'a> {
    pub const fn new(storage_key: &'a str) -> Self {
        UserMintedTokens(Map::new(storage_key))
    }

    pub fn load(
        &self,
        store: &dyn Storage,
        drop_id: DropID,
        address: Addr,
    ) -> Result<UserDetails, StdError> {
        let user_details = self.0.load(store, (drop_id, address))?;
        Ok(user_details)
    }

    pub fn save(
        &self,
        store: &mut dyn Storage,
        drop_id: DropID,
        address: Addr,
        data: &UserDetails,
    ) {
        self.0.save(store, (drop_id, address), data).unwrap();
    }
}
