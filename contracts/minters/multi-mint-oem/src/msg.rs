use crate::mint_instance::MintInstance;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Coin;
use minter_types::{
    config::Config, msg::MintHistoryResponse, token_details::TokenDetails, types::UserDetails,
};
use omniflix_std::types::omniflix::onft::v1beta1::WeightedAddress;

#[cw_serde]
pub enum ExecuteMsg {
    Mint {
        mint_instance_id: Option<u32>,
    },
    MintAdmin {
        recipient: String,
        mint_instance_id: Option<u32>,
    },
    UpdateRoyaltyRatio {
        ratio: String,
        mint_instance_id: Option<u32>,
    },
    UpdateMintPrice {
        mint_price: Coin,
        mint_instance_id: Option<u32>,
    },
    UpdateWhitelistAddress {
        address: String,
        mint_instance_id: Option<u32>,
    },
    Pause {},
    Unpause {},
    SetPausers {
        pausers: Vec<String>,
    },
    NewMintInstance {
        token_details: TokenDetails,
        config: Config,
    },
    RemoveMintInstance {
        mint_instance_id: u32,
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
    TokenDetails { mint_instance_id: Option<u32> },
    #[returns(Config)]
    Config { mint_instance_id: Option<u32> },
    #[returns(UserDetails)]
    UserMintingDetails {
        address: String,
        mint_instance_id: Option<u32>,
    },
    #[returns(u32)]
    TokensRemainingInMintInstance { mint_instance_id: Option<u32> },
    #[returns(u32)]
    TokensMintedInMintInstance { mint_instance_id: Option<u32> },
    #[returns(u32)]
    ActiveMintInstanceId {},
    #[returns(Vec<(u32,MintInstance)>)]
    AllMintInstances {},
    #[returns(MintHistoryResponse)]
    MintHistory {
        address: String,
        mint_instance_id: Option<u32>,
    },
}
