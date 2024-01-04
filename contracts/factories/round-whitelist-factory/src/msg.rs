use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Coin, Timestamp, Uint128};
use whitelist_types::InstantiateMsg as WhitelistInstantiateMsg;

use crate::state::Params;

#[cw_serde]
pub struct InstantiateMsg {
    pub admin: Option<String>,
    pub fee_collector_address: String,
    pub whitelist_code_id: u64,
    pub whitelist_creation_fee: Coin,
}

#[cw_serde]
pub enum ExecuteMsg {
    CreateWhitelist { msg: WhitelistInstantiateMsg },
    UpdateAdmin { admin: String },
    UpdateFeeCollectorAddress { fee_collector_address: String },
    UpdateWhitelistCreationFee { whitelist_creation_fee: Coin },
    UpdateWhitelistCodeId { whitelist_code_id: u64 },
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
