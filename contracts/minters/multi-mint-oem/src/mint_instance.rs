use cosmwasm_schema::cw_serde;
use cosmwasm_std::{StdError, Storage};
use cw_storage_plus::{Item, Map};
use minter_types::config::Config;
use minter_types::token_details::TokenDetails;
use std::u32;

use crate::error::ContractError;

pub type MintInstanceID = u32;

#[cw_serde]
pub struct MintInstanceParams {
    pub config: Config,
    pub token_details: TokenDetails,
}

#[cw_serde]
pub struct MintInstance {
    pub minted_count: u32,
    pub mint_instance_params: MintInstanceParams,
}

pub const MINT_INSTANCES: Map<MintInstanceID, MintInstance> = Map::new("mint_instances");

pub const ACTIVE_MINT_INSTANCE_ID: Item<MintInstanceID> = Item::new("active_mint_instance_id");

pub const MINT_INSTANCE_IDS_IN_USE: Item<Vec<MintInstanceID>> =
    Item::new("mint_instance_ids_in_use");

pub const MINT_INSTANCE_IDS_REMOVED: Item<Vec<MintInstanceID>> =
    Item::new("mint_instance_ids_removed");

pub fn return_latest_mint_instance_id(store: &dyn Storage) -> Result<u32, StdError> {
    // Returns the latest mint_instance id.
    // Combines the mint_instance ids in use and the mint_instance ids removed to find the maximum id
    // Its used to generate the next mint_instance id
    let mint_instance_ids_in_use = MINT_INSTANCE_IDS_IN_USE.load(store).unwrap_or_default();
    let mint_instance_ids_removed = MINT_INSTANCE_IDS_REMOVED.load(store).unwrap_or_default();

    // Merge the two lists and find the maximum id
    let max_id = mint_instance_ids_in_use
        .iter()
        .chain(mint_instance_ids_removed.iter())
        .cloned()
        .max()
        .unwrap_or(0); // If no ids are found, default to 0

    Ok(max_id)
}
pub fn return_latest_mint_instance_id_in_use(store: &dyn Storage) -> Result<u32, StdError> {
    // Returns the latest mint_instance id in use.
    // Its used to find active mint_instance id
    let mint_instance_ids_in_use = MINT_INSTANCE_IDS_IN_USE.load(store).unwrap_or_default();

    // Find the maximum id
    let max_id = mint_instance_ids_in_use.iter().cloned().max().unwrap_or(0);

    Ok(max_id)
}

pub fn get_mint_instance_by_id(
    mint_instance_id: Option<MintInstanceID>,
    store: &dyn Storage,
) -> Result<(MintInstanceID, MintInstance), ContractError> {
    let mint_instance_id = mint_instance_id.unwrap_or(ACTIVE_MINT_INSTANCE_ID.load(store)?);
    if mint_instance_id == 0 {
        return Err(ContractError::NoMintInstanceAvailable {});
    }
    let mint_instance = MINT_INSTANCES
        .load(store, mint_instance_id)
        .map_err(|_| ContractError::InvalidMintInstanceId {})?;
    Ok((mint_instance_id, mint_instance))
}
