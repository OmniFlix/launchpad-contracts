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
    UpdateParams {
        admin: Option<String>,
        allowed_mint_denoms: Option<Vec<String>>,
        fee_collector_address: Option<String>,
        minter_code_id: Option<u64>,
        minter_creation_fee: Option<Coin>,
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
