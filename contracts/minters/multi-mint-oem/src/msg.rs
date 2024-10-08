use crate::drop::Drop;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Coin;
use minter_types::{
    config::Config, msg::MintHistoryResponse, token_details::TokenDetails, types::UserDetails,
};
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
    RemoveDrop {
        drop_id: u32,
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
    UpdateAdmin {
        admin: String,
    },
    UpdatePaymentCollector {
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
    TokensMintedInDrop { drop_id: Option<u32> },
    #[returns(u32)]
    ActiveDropId {},
    #[returns(Vec<(u32,Drop)>)]
    AllDrops {},
    #[returns(MintHistoryResponse)]
    MintHistory {
        address: String,
        drop_id: Option<u32>,
    },
}
