use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Coin, Empty, Timestamp};
use factory_types::FactoryParams;
use minter_types::MinterInstantiateMsg;
#[cw_serde]
pub struct InstantiateMsg {
    pub params: FactoryParams<Empty>,
}

#[cw_serde]
pub struct OpenEditionMinterInitExtention {
    pub admin: String,
    pub mint_price: Coin,
    pub start_time: Timestamp,
    pub end_time: Option<Timestamp>,
    pub num_tokens: Option<u32>,
    pub per_address_limit: u32,
    pub payment_collector: Option<String>,
    pub whitelist_address: Option<String>,
}

pub type OpenEditionMinterCreateMsg = MinterInstantiateMsg<OpenEditionMinterInitExtention>;

#[cw_serde]
pub enum ExecuteMsg {
    CreateMinter { msg: OpenEditionMinterCreateMsg },
    UpdateAdmin { admin: String },
    UpdateFeeCollectorAddress { fee_collector_address: String },
    UpdateMinterCreationFee { minter_creation_fee: Coin },
    UpdateMinterCodeId { minter_code_id: u64 },
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
