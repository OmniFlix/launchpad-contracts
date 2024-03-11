use crate::types::{AuthDetails, CollectionDetails, Config, TokenDetails, UserDetails};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Addr;
#[cw_serde]
pub struct MinterInstantiateMsg<T> {
    pub collection_details: CollectionDetails,
    pub token_details: Option<TokenDetails>,
    pub init: T,
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
}
