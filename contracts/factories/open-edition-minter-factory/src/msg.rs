use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Coin, Empty, Timestamp};
use minter_types::msg::MinterInstantiateMsg;
#[cw_serde]
pub struct InstantiateMsg {
    pub params: OpenEditionMinterFactoryParams,
}

#[cw_serde]
pub struct OpenEditionMinterInitExtention {
    pub mint_price: Coin,
    pub start_time: Timestamp,
    pub end_time: Option<Timestamp>,
    pub num_tokens: Option<u32>,
    pub per_address_limit: Option<u32>,
    pub whitelist_address: Option<String>,
}

pub type OpenEditionMinterCreateMsg = MinterInstantiateMsg<OpenEditionMinterInitExtention>;
pub type MultiMinterCreateMsg = MinterInstantiateMsg<Empty>;

#[allow(clippy::large_enum_variant)]
#[cw_serde]
pub enum ExecuteMsg {
    CreateOpenEditionMinter {
        msg: OpenEditionMinterCreateMsg,
    },
    CreateMultiMintOpenEditionMinter {
        msg: MultiMinterCreateMsg,
    },
    UpdateAdmin {
        admin: String,
    },
    UpdateFeeCollectorAddress {
        fee_collector_address: String,
    },
    UpdateOpenEditionMinterCreationFee {
        open_edition_minter_creation_fee: Coin,
    },
    UpdateOpenEditionMinterCodeId {
        open_edition_minter_code_id: u64,
    },
    UpdateMultiMinterCreationFee {
        multi_minter_creation_fee: Coin,
    },
    UpdateMultiMinterCodeId {
        multi_minter_code_id: u64,
    },
    Pause {},
    Unpause {},
    SetPausers {
        pausers: Vec<String>,
    },
}

#[cw_serde]
pub struct ParamsResponse {
    pub params: OpenEditionMinterFactoryParams,
}
#[cw_serde]
pub struct MultiMinterParams {
    pub multi_minter_code_id: u64,
    pub multi_minter_creation_fee: Coin,
    pub multi_minter_product_label: String,
}
#[cw_serde]
pub struct OpenEditionMinterFactoryParams {
    pub open_edition_minter_code_id: u64,
    pub open_edition_minter_creation_fee: Coin,
    pub fee_collector_address: Addr,
    pub admin: Addr,
    pub oem_product_label: String,
    pub multi_minter_params: Option<MultiMinterParams>,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ParamsResponse)]
    Params {},
    #[returns(bool)]
    IsPaused {},
    #[returns(Vec<Addr>)]
    Pausers {},
}
