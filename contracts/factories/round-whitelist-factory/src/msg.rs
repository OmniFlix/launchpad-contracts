use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Coin, Empty};
use factory_types::FactoryParams;
use whitelist_types::InstantiateMsg as WhitelistInstantiateMsg;

#[cw_serde]
pub struct InstantiateMsg {
    pub params: FactoryParams<Empty>,
}
#[allow(clippy::large_enum_variant)]
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
    pub params: FactoryParams<Empty>,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ParamsResponse)]
    Params {},
}
