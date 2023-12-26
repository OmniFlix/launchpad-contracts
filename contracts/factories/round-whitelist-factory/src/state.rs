use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Coin};
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct Params {
    pub minter_creation_fee: Coin,
    pub fee_collector_address: Addr,
    pub minter_code_id: u64,
    pub allowed_minter_mint_denoms: Vec<String>,
    pub admin: Addr,
}
