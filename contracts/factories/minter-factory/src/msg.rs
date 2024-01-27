use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Coin, Timestamp};
use minter_types::MinterInstantiateMsg;

use crate::state::Params;
#[cw_serde]
pub struct InstantiateMsg {
    pub admin: Option<String>,
    pub fee_collector_address: String,
    pub minter_code_id: u64,
    pub minter_creation_fee: Coin,
}

#[cw_serde]
pub struct MinterInitExtention {
    pub admin: String,
    pub mint_price: Coin,
    // Public minting start time
    pub start_time: Timestamp,
    pub end_time: Option<Timestamp>,
    pub per_address_limit: u32,
    // We expect user to send a string between 0 and 1
    // FE "0.1"
    pub royalty_ratio: String,
    pub payment_collector: Option<String>,
    // Whitelist address if any
    pub whitelist_address: Option<String>,
    pub num_tokens: u32,
}

pub type CreateMinterMsg = MinterInstantiateMsg<MinterInitExtention>;

#[cw_serde]
pub enum ExecuteMsg {
    CreateMinter { msg: CreateMinterMsg },
    UpdateAdmin { admin: String },
    UpdateFeeCollectorAddress { fee_collector_address: String },
    UpdateMinterCreationFee { minter_creation_fee: Coin },
    UpdateMinterCodeId { minter_code_id: u64 },
}

#[cw_serde]
pub struct ParamsResponse {
    pub params: Params,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ParamsResponse)]
    Params {},
}
