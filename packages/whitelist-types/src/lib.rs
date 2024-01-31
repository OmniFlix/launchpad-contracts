use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::StdError;
use cosmwasm_std::{Addr, Coin, Deps, Storage, Timestamp};
use minter_types::Config as MinterConfig;
use minter_types::QueryMsg as MinterQueryMsg;

#[cw_serde]
pub struct InstantiateMsg {
    pub admin: Option<String>,
    pub rounds: Vec<Round>,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum RoundWhitelistQueryMsgs {
    #[returns(Vec<(u32,Round)>)]
    Rounds {},

    #[returns(Round)]
    Round { round_index: u32 },
    // Returns true if any round is active
    #[returns(IsActiveResponse)]
    IsActive {},

    #[returns((u32,Round))]
    ActiveRound {},

    #[returns(MembersResponse)]
    Members {
        round_index: u32,
        start_after: Option<String>,
        limit: Option<u32>,
    },
    // Returns price of the active round
    #[returns(MintPriceResponse)]
    Price {},

    #[returns(IsMemberResponse)]
    IsMember { address: String },

    #[returns(String)]
    Admin {},
}
#[cw_serde]
pub struct MembersResponse {
    pub members: Vec<String>,
}

#[cw_serde]
pub struct IsMemberResponse {
    pub is_member: bool,
}

#[cw_serde]
pub struct IsActiveResponse {
    pub is_active: bool,
}

#[cw_serde]
pub struct MintPriceResponse {
    pub mint_price: Coin,
}

#[cw_serde]
pub struct Round {
    pub addresses: Vec<Addr>,
    pub start_time: Timestamp,
    pub end_time: Timestamp,
    pub mint_price: Coin,
    pub round_per_address_limit: u32,
}

pub fn check_if_minter(address: &Addr, deps: Deps) -> Result<(), StdError> {
    // Check if sender is a minter contract
    let _minter_config: MinterConfig = deps
        .querier
        .query_wasm_smart(address, &MinterQueryMsg::Config {})?;
    Ok(())
}

pub fn check_if_whitelist_is_active(address: &Addr, deps: Deps) -> Result<bool, StdError> {
    let is_active_res: IsActiveResponse = deps
        .querier
        .query_wasm_smart(address, &RoundWhitelistQueryMsgs::IsActive {})?;
    Ok(is_active_res.is_active)
}

pub fn check_if_address_is_member(address: &Addr, deps: Deps) -> Result<bool, StdError> {
    let is_member_res: IsMemberResponse = deps.querier.query_wasm_smart(
        address,
        &RoundWhitelistQueryMsgs::IsMember {
            address: address.to_string(),
        },
    )?;
    Ok(is_member_res.is_member)
}

pub fn check_whitelist_price(address: &Addr, deps: Deps) -> Result<Coin, StdError> {
    let price_res: MintPriceResponse = deps
        .querier
        .query_wasm_smart(address, &RoundWhitelistQueryMsgs::Price {})?;
    Ok(price_res.mint_price)
}
