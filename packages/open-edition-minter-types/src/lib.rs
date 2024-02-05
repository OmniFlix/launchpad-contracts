use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Addr;
use minter_types::{CollectionDetails, Config, UserDetails};

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(CollectionDetails)]
    Collection { edition: Option<u32> },
    #[returns(Config)]
    Config { edition: Option<u32> },
    #[returns(UserDetails)]
    MintedTokens {
        address: String,
        edition: Option<u32>,
    },
    #[returns(u32)]
    TotalMintedCount { edition: Option<u32> },
    #[returns(u32)]
    TokensRemaining { edition: Option<u32> },
    #[returns(bool)]
    IsPaused {},
    #[returns(Vec<Addr>)]
    Pausers {},
}
