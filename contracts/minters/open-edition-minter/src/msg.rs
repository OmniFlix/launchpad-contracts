use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Coin, Timestamp};

#[cw_serde]
pub enum ExecuteMsg {
    Mint {},
    MintAdmin {
        recipient: String,
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
        whitelist_address: Option<String>,
        token_limit: Option<u32>,
        start_time: Timestamp,
        end_time: Option<Timestamp>,
        mint_price: Coin,
        royalty_ratio: String,
        token_name: String,
        description: String,
        base_uri: String,
        preview_uri: String,
        uri_hash: String,
        transferable: bool,
        extensible: bool,
        nsfw: bool,
        data: String,
    },
}
