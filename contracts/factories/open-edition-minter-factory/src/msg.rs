use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Coin, Timestamp, Uint128};
use minter_types::MinterInstantiateMsg;

use crate::state::Params;
#[cw_serde]
pub struct InstantiateMsg {
    pub admin: Option<String>,
    pub allowed_minter_mint_denoms: Vec<String>,
    pub fee_collector_address: String,
    pub open_edition_minter_code_id: u64,
    pub minter_creation_fee: Coin,
}

#[cw_serde]
pub struct OpenEditionMinterInitExtention {
    pub admin: String,
    pub mint_price: Uint128,
    pub mint_denom: String,
    pub start_time: Timestamp,
    pub end_time: Option<Timestamp>,
    pub token_limit: Option<u32>,
    pub per_address_limit: u32,
    pub royalty_ratio: String,
    pub payment_collector: Option<String>,
    pub whitelist_address: Option<String>,
}

pub type OpenEditionMinterCreateMsg = MinterInstantiateMsg<OpenEditionMinterInitExtention>;

#[cw_serde]
pub enum ExecuteMsg {
    CreateMinter {
        msg: OpenEditionMinterCreateMsg,
    },
    UpdateAdmin {
        admin: String,
    },
    UpdateFeeCollectorAddress {
        fee_collector_address: String,
    },
    UpdateMinterCreationFee {
        minter_creation_fee: Coin,
    },
    UpdateAllowedMinterMintDenoms {
        allowed_minter_mint_denoms: Vec<String>,
    },
    UpdateMinterCodeId {
        minter_code_id: u64,
    },
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
