use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Coin, Timestamp};
use minter_types::MinterInstantiateMsg;
#[cw_serde]
pub struct InstantiateMsg {
    pub params: MinterFactoryParams,
}

#[cw_serde]
pub struct MinterInitExtention {
    pub admin: String,
    pub mint_price: Coin,
    // Public minting start time
    pub start_time: Timestamp,
    pub end_time: Option<Timestamp>,
    pub per_address_limit: Option<u32>,
    // We expect user to send a string between 0 and 1
    // FE "0.1"
    pub payment_collector: Option<String>,
    // Whitelist address if any
    pub whitelist_address: Option<String>,
    pub num_tokens: u32,
}

pub type CreateMinterMsg = MinterInstantiateMsg<MinterInitExtention>;
#[allow(clippy::large_enum_variant)]
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
    pub params: MinterFactoryParams,
}
#[cw_serde]
pub struct MinterFactoryParams {
    pub minter_code_id: u64,
    pub minter_creation_fee: Coin,
    pub fee_collector_address: Addr,
    pub admin: Addr,
    pub product_label: String,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ParamsResponse)]
    Params {},
}
