use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Coin, Timestamp, Uint128};
use minter_types::InstantiateMsg as MinterInstantiateMsg;

use crate::state::Params;
#[cw_serde]
pub struct InstantiateMsg {
    pub admin: Option<String>,
    pub allowed_minter_mint_denoms: Vec<String>,
    pub fee_collector_address: String,
    pub minter_code_id: u64,
    pub minter_creation_fee: Coin,
}
#[cw_serde]
pub enum ExecuteMsg {
    CreateMinter {
        msg: MinterInstantiateMsg,
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
    params: Params,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ParamsResponse)]
    Params {},
}
