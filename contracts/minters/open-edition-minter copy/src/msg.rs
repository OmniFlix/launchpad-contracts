use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Coin, Timestamp};
use omniflix_std::types::omniflix::onft::v1beta1::WeightedAddress;

#[cw_serde]
pub enum ExecuteMsg {
    Mint {
        edition: Option<u32>,
    },
    MintAdmin {
        recipient: String,
        edition: Option<u32>,
    },
    UpdateRoyaltyRatio {
        ratio: String,
    },
    UpdateMintPrice {
        mint_price: Coin,
    },
    UpdateWhitelistAddress {
        address: String,
    },
    Pause {},
    Unpause {},
    SetPausers {
        pausers: Vec<String>,
    },
    NewEdition {
        start_time: Timestamp,
        mint_price: Coin,
        token_name: String,
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
