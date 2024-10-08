use crate::{
    collection_details::CollectionDetails,
    config::Config,
    token_details::TokenDetails,
    types::{AuthDetails, UserDetails},
};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Addr;

#[cw_serde]
pub struct MinterInstantiateMsg<T> {
    pub collection_details: CollectionDetails,
    pub token_details: Option<TokenDetails>,
    pub auth_details: AuthDetails,
    pub init: Option<T>,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg<T> {
    #[returns(CollectionDetails)]
    Collection {},
    #[returns(TokenDetails)]
    TokenDetails {},
    #[returns(AuthDetails)]
    AuthDetails {},
    #[returns(Config)]
    Config {},
    #[returns(UserDetails)]
    UserMintingDetails { address: String },
    #[returns(bool)]
    IsPaused {},
    #[returns(Vec<Addr>)]
    Pausers {},
    #[returns(u32)]
    Extension(T),
    #[returns(u32)]
    TotalMintedCount {},
    #[returns(MintHistoryResponse)]
    MintHistory { address: String },
}

#[cw_serde]
pub struct MintHistoryResponse {
    pub public_minted_count: u32,
    pub public_mint_limit: u32,
    pub total_minted_count: u32,
}
