use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Coin, Timestamp};
use minter_types::{
    CollectionDetails, Config, QueryMsg as BaseMinterQueryMsg, TokenDetails, UserDetails,
};
use multi_mint_open_edition_minter_types::DropParams;
use omniflix_std::types::omniflix::onft::v1beta1::WeightedAddress;

#[cw_serde]
pub enum ExecuteMsg {
    Mint {
        drop_id: Option<u32>,
    },
    MintAdmin {
        recipient: String,
        drop_id: Option<u32>,
    },
    UpdateRoyaltyRatio {
        ratio: String,
        drop_id: Option<u32>,
    },
    UpdateMintPrice {
        mint_price: Coin,
        drop_id: Option<u32>,
    },
    UpdateWhitelistAddress {
        address: String,
        drop_id: Option<u32>,
    },
    Pause {},
    Unpause {},
    SetPausers {
        pausers: Vec<String>,
    },
    NewDrop {
        new_token_details: TokenDetails,
        new_config: Config,
    },
    UpdateRoyaltyReceivers {
        receivers: Vec<WeightedAddress>,
    },
    UpdateDenom {
        name: Option<String>,
        description: Option<String>,
        preview_uri: Option<String>,
    },
    PurgeDenom {},
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsgExtension {
    #[returns(CollectionDetails)]
    Collection { drop_id: Option<u32> },
    #[returns(TokenDetails)]
    TokenDetails { drop_id: Option<u32> },
    #[returns(Config)]
    Config { drop_id: Option<u32> },
    #[returns(UserDetails)]
    MintedTokens {
        address: String,
        drop_id: Option<u32>,
    },
    #[returns(u32)]
    TokensRemaining { drop_id: Option<u32> },
    #[returns(u32)]
    CurrentDropNumber {},
    #[returns(Vec<(u32,DropParams)>)]
    AllDrops {},
}
