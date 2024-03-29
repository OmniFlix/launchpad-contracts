use cosmwasm_schema::cw_serde;
use cosmwasm_std::{StdError, Storage};
use cw_storage_plus::{Item, Map};
use minter_types::config::Config;
use minter_types::token_details::TokenDetails;
use std::u32;

use crate::error::ContractError;

pub type DropID = u32;

#[cw_serde]
pub struct DropParams {
    pub config: Config,
    pub token_details: TokenDetails,
}

#[cw_serde]
pub struct Drop {
    pub minted_count: u32,
    pub drop_params: DropParams,
}

pub const DROPS: Map<DropID, Drop> = Map::new("drops");

pub const ACTIVE_DROP_ID: Item<DropID> = Item::new("active_drop_id");

pub const DROP_IDS_IN_USE: Item<Vec<DropID>> = Item::new("drop_ids_in_use");

pub const DROP_IDS_REMOVED: Item<Vec<DropID>> = Item::new("drop_ids_removed");

pub fn return_latest_drop_id(store: &dyn Storage) -> Result<u32, StdError> {
    // Returns the latest drop id.
    // Combines the drop ids in use and the drop ids removed to find the maximum id
    // Its used to generate the next drop id
    let drop_ids_in_use = DROP_IDS_IN_USE.load(store).unwrap_or_default();
    let drop_ids_removed = DROP_IDS_REMOVED.load(store).unwrap_or_default();

    // Merge the two lists and find the maximum id
    let max_id = drop_ids_in_use
        .iter()
        .chain(drop_ids_removed.iter())
        .cloned()
        .max()
        .unwrap_or(0); // If no ids are found, default to 0

    Ok(max_id)
}
pub fn return_latest_drop_id_in_use(store: &dyn Storage) -> Result<u32, StdError> {
    // Returns the latest drop id in use.
    // Its used to find active drop id
    let drop_ids_in_use = DROP_IDS_IN_USE.load(store).unwrap_or_default();

    // Find the maximum id
    let max_id = drop_ids_in_use.iter().cloned().max().unwrap_or(0);

    Ok(max_id)
}

pub fn get_drop_by_id(
    drop_id: Option<DropID>,
    store: &dyn Storage,
) -> Result<(DropID, Drop), ContractError> {
    let drop_id = drop_id.unwrap_or(ACTIVE_DROP_ID.load(store)?);
    if drop_id == 0 {
        return Err(ContractError::NoDropAvailable {});
    }
    let drop = DROPS
        .load(store, drop_id)
        .map_err(|_| ContractError::InvalidDropId {})?;
    Ok((drop_id, drop))
}
