use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Coin, Timestamp};
use factory_types::FactoryParams;
use minter_types::MinterInstantiateMsg;
#[cw_serde]
pub struct InstantiateMsg {
    pub params: FactoryParams<MultiMinterFactoryExtension>,
}
#[cw_serde]
pub struct MultiMinterFactoryExtension {
    pub multi_minter_code_id: u64,
    pub multi_minter_creation_fee: Coin,
}

#[cw_serde]
pub struct OpenEditionMinterInitExtention {
    pub admin: String,
    pub mint_price: Coin,
    pub start_time: Timestamp,
    pub end_time: Option<Timestamp>,
    pub num_tokens: Option<u32>,
    pub per_address_limit: Option<u32>,
    pub payment_collector: Option<String>,
    pub whitelist_address: Option<String>,
}
#[cw_serde]
pub struct MultiMinterInitExtention {
    pub admin: String,
    pub payment_collector: Option<String>,
}

pub type OpenEditionMinterCreateMsg = MinterInstantiateMsg<OpenEditionMinterInitExtention>;
pub type MultiMinterCreateMsg = MinterInstantiateMsg<MultiMinterInitExtention>;

#[allow(clippy::large_enum_variant)]
#[cw_serde]
pub enum ExecuteMsg {
    CreateOpenEditionMinter { msg: OpenEditionMinterCreateMsg },
    CreateMultiMintOpenEditionMinter { msg: MultiMinterCreateMsg },
    UpdateAdmin { admin: String },
    UpdateFeeCollectorAddress { fee_collector_address: String },
    UpdateMinterCreationFee { minter_creation_fee: Coin },
    UpdateMinterCodeId { minter_code_id: u64 },
    UpdateMultiMinterCreationFee { multi_minter_creation_fee: Coin },
    UpdateMultiMinterCodeId { multi_minter_code_id: u64 },
}

#[cw_serde]
pub struct ParamsResponse {
    pub params: FactoryParams<MultiMinterFactoryExtension>,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ParamsResponse)]
    Params {},
}
