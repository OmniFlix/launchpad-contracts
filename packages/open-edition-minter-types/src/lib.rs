use cosmwasm_schema::{cw_serde, QueryResponses};
use minter_types::{CollectionDetails, Config, UserDetails};

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(CollectionDetails)]
    Collection {},
    #[returns(Config)]
    Config {},
    #[returns(UserDetails)]
    MintedTokens { address: String },
    #[returns(u32)]
    TotalMintedCount {},
    #[returns(u32)]
    TokensRemaining {},
}
