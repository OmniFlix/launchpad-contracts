use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Coin, Timestamp};
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct Config {
    pub admin: Addr,
    pub start_time: Timestamp,
    pub end_time: Timestamp,
    pub mint_price: Coin,
    pub per_address_limit: u32,
    pub member_limit: u32,
    pub is_frozen: bool,
}

pub const MEMBERS: Map<Addr, bool> = Map::new("members");
pub const CONFIG: Item<Config> = Item::new("config");
