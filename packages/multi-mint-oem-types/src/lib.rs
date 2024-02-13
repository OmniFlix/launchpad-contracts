use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Addr;
use minter_types::{CollectionDetails, Config, UserDetails};

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(CollectionDetails)]
    Collection { drop_id: Option<u32> },
    #[returns(Config)]
    Config { drop_id: Option<u32> },
    #[returns(UserDetails)]
    MintedTokens {
        address: String,
        drop_id: Option<u32>,
    },
    #[returns(u32)]
    TotalMintedCount { drop_id: Option<u32> },
    #[returns(u32)]
    TokensRemaining { drop_id: Option<u32> },
    #[returns(bool)]
    IsPaused {},
    #[returns(Vec<Addr>)]
    Pausers {},
    #[returns(u32)]
    CurrentDropNumber {},
}
