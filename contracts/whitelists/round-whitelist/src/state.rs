use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Coin, Timestamp};

#[cw_serde]
pub enum Round {
    WhitelistAddresses {
        addresses: Vec<Addr>,
        start_time: Timestamp,
        end_time: Timestamp,
        mint_price: Coin,
        round_per_address_limit: u32,
    },
    WhitelistCollection {
        collection_id: String,
        start_time: Timestamp,
        end_time: Timestamp,
        mint_price: Coin,
        round_per_address_limit: u32,
    },
}
