use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Coin, Timestamp};
use minter_types::{CollectionDetails, Config, QueryMsg as BaseMinterQueryMsg, UserDetails};
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
        start_time: Timestamp,
        mint_price: Coin,
        token_name: String,
        per_address_limit: u32,
        token_limit: Option<u32>,
        whitelist_address: Option<String>,
        end_time: Option<Timestamp>,
        royalty_ratio: Option<String>,
        description: Option<String>,
        base_uri: Option<String>,
        preview_uri: Option<String>,
        uri_hash: Option<String>,
        transferable: Option<bool>,
        extensible: Option<bool>,
        nsfw: Option<bool>,
        data: Option<String>,
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
pub enum QueryMsgExtention {
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
    TokensRemaining { drop_id: Option<u32> },
    #[returns(u32)]
    CurrentDropNumber {},
    #[returns(Vec<(u32,DropParams)>)]
    AllDrops {},
}
