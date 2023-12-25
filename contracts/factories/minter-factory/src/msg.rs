use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Coin, Timestamp, Uint128};
pub struct InstantiateMsg {
    pub admin: Option<String>,
    pub allowed_minter_mint_denoms: Vec<String>,
    pub fee_collector_address: String,
    pub minter_code_id: u64,
    pub minter_creation_fee: Coin,
}
