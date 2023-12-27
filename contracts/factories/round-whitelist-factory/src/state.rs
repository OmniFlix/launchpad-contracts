use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Coin};
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct Params {
    pub whitelist_creation_fee: Coin,
    pub fee_collector_address: Addr,
    pub whitelist_code_id: u64,
    pub admin: Addr,
}
pub const PARAMS: Item<Params> = Item::new("params");
