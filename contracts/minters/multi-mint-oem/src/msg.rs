use crate::state::DropParams;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Coin;
use minter_types::{Config, TokenDetails, UserDetails};
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
        token_details: TokenDetails,
        config: Config,
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
    SetAdmin {
        admin: String,
    },
    SetPaymentCollector {
        payment_collector: String,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsgExtension {
    #[returns(TokenDetails)]
    TokenDetails { drop_id: Option<u32> },
    #[returns(Config)]
    Config { drop_id: Option<u32> },
    #[returns(UserDetails)]
    UserMintingDetails {
        address: String,
        drop_id: Option<u32>,
    },
    #[returns(u32)]
    TokensRemainingInDrop { drop_id: Option<u32> },
    #[returns(u32)]
    TotalTokensMintedInDrop { drop_id: Option<u32> },
    #[returns(u32)]
    CurrentDropNumber {},
    #[returns(Vec<(u32,DropParams)>)]
    AllDrops {},
}
