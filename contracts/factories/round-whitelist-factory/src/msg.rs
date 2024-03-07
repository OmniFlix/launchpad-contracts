use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Coin};
use whitelist_types::InstantiateMsg as WhitelistInstantiateMsg;

#[cw_serde]
pub struct InstantiateMsg {
    pub params: RoundWhitelistFactoryParams,
}

#[cw_serde]
pub struct RoundWhitelistFactoryParams {
    pub whitelist_code_id: u64,
    pub whitelist_creation_fee: Coin,
    pub fee_collector_address: Addr,
    pub admin: Addr,
    pub product_label: String,
}
#[allow(clippy::large_enum_variant)]
#[cw_serde]
pub enum ExecuteMsg {
    CreateWhitelist { msg: WhitelistInstantiateMsg },
    UpdateAdmin { admin: String },
    UpdateFeeCollectorAddress { fee_collector_address: String },
    UpdateWhitelistCreationFee { whitelist_creation_fee: Coin },
    UpdateWhitelistCodeId { whitelist_code_id: u64 },
    Pause {},
    Unpause {},
    SetPausers { pausers: Vec<String> },
}

#[cw_serde]
pub struct ParamsResponse {
    pub params: RoundWhitelistFactoryParams,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ParamsResponse)]
    Params {},
}
