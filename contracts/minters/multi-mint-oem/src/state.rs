use crate::drop::DropID;
use cosmwasm_std::{Addr, StdError, Storage};
use cw_storage_plus::{Item, Map};
use minter_types::{
    collection_details::CollectionDetails,
    types::{AuthDetails, UserDetails},
};
use std::u32;
// Last token id minted.
// It is used to generate the next token id. Seperate from a drop to allow for multiple drops to mint tokens.
pub const LAST_MINTED_TOKEN_ID: Item<u32> = Item::new("last_minted_token_id");
pub const AUTH_DETAILS: Item<AuthDetails> = Item::new("auth_details");
pub const COLLECTION: Item<CollectionDetails> = Item::new("collection");

pub const USER_MINTING_DETAILS_KEY: &str = "user_minting_details";
pub struct UserMintingDetails<'a>(Map<'a, (DropID, Addr), UserDetails>);

impl<'a> UserMintingDetails<'a> {
    pub const fn new(storage_key: &'a str) -> Self {
        UserMintingDetails(Map::new(storage_key))
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
