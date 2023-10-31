use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Coin, Timestamp};
use cw_storage_plus::{Item, Map};
use types::whitelist::Config;

pub const MEMBERS: Map<Addr, bool> = Map::new("members");
pub const CONFIG: Item<Config> = Item::new("config");
