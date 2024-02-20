use cosmwasm_schema::cw_serde;
use minter_types::{CollectionDetails, Config};

#[cw_serde]
pub struct DropParams {
    pub config: Config,
    pub collection: CollectionDetails,
}
