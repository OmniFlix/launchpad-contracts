use std::u32;

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Coin, Decimal, Timestamp, Uint128};
use cw_storage_plus::{Item, Map};

use crate::{error::ContractError, msg::CollectionDetails};

#[cw_serde]
pub struct Config {
    pub per_address_limit: u32,
    pub payment_collector: Addr,
    pub start_time: Timestamp,
    pub mint_price: Coin,
    pub royalty_ratio: Decimal,
    pub admin: Addr,
}

#[cw_serde]
pub struct Token {
    pub token_id: String,
}

// impl Round {
//     //     pub fn start_time(&self) -> Timestamp {
//     //         match self {
//     //             Round::WhitelistAddress { start_time, .. } => start_time.unwrap(),
//     //             Round::WhitelistCollection { start_time, .. } => *start_time,
//     //         }
//     //     }
//     //     pub fn end_time(&self) -> Timestamp {
//     //         match self {
//     //             Round::WhitelistAddress { end_time, .. } => end_time.unwrap(),
//     //             Round::WhitelistCollection { end_time, .. } => *end_time,
//     //         }
//     //     }
//     //     pub fn mint_price(&self) -> Uint128 {
//     //         match self {
//     //             Round::WhitelistAddress { mint_price, .. } => *mint_price,
//     //             Round::WhitelistCollection { mint_price, .. } => *mint_price,
//     //         }
//     //     }
//     //     pub fn round_limit(&self) -> u32 {
//     //         match self {
//     //             Round::WhitelistAddress { round_limit, .. } => *round_limit,
//     //             Round::WhitelistCollection { round_limit, .. } => *round_limit,
//     //         }
//     //     }
//     //     pub fn return_whitelist_address(&self) -> Option<Addr> {
//     //         match self {
//     //             Round::WhitelistAddress { address, .. } => Some(address.clone()),
//     //             Round::WhitelistCollection { .. } => None,
//     //         }
//     //     }
//     pub fn update_params(
//         &mut self,
//         start_time: Option<Timestamp>,
//         end_time: Option<Timestamp>,
//         mint_price: Option<Uint128>,
//         round_limit: Option<u32>,
//     ) -> Result<(), ContractError> {
//         match self {
//             Round::WhitelistAddress {
//                 start_time: ref mut s,
//                 end_time: ref mut e,
//                 mint_price: ref mut m,
//                 round_limit: ref mut r,
//                 ..
//             } => {
//                 if let Some(start_time) = start_time {
//                     *s = Some(start_time);
//                 }
//                 if let Some(end_time) = end_time {
//                     *e = Some(end_time);
//                 }
//                 if let Some(mint_price) = mint_price {
//                     *m = mint_price;
//                 }
//                 if let Some(round_limit) = round_limit {
//                     *r = round_limit;
//                 }
//             }
//             Round::WhitelistCollection {
//                 start_time: ref mut s,
//                 end_time: ref mut e,
//                 mint_price: ref mut m,
//                 round_limit: ref mut r,
//                 ..
//             } => {
//                 if let Some(start_time) = start_time {
//                     *s = start_time;
//                 }
//                 if let Some(end_time) = end_time {
//                     *e = end_time;
//                 }
//                 if let Some(mint_price) = mint_price {
//                     *m = mint_price;
//                 }
//                 if let Some(round_limit) = round_limit {
//                     *r = round_limit;
//                 }
//             }
//         }
//         Ok(())
//     }
//     pub fn return_round_type(&self) -> String {
//         match self {
//             Round::WhitelistAddress { .. } => "address".to_string(),
//             Round::WhitelistCollection { .. } => "collection".to_string(),
//         }
//     }
// }

#[cw_serde]
pub struct UserDetails {
    pub minted_tokens: Vec<Token>,
    pub total_minted_count: u32,
}

impl UserDetails {
    pub fn new() -> Self {
        UserDetails {
            minted_tokens: Vec::new(),
            total_minted_count: 0,
        }
    }
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const COLLECTION: Item<CollectionDetails> = Item::new("collection");
// Index of mintable tokens and denom ids
pub const MINTABLE_TOKENS: Map<u32, Token> = Map::new("mintable_tokens");
// Total number of tokens
pub const TOTAL_TOKENS_REMAINING: Item<u32> = Item::new("total_tokens_remaining");
// Address and number of tokens minted
pub const MINTED_TOKENS: Map<Addr, UserDetails> = Map::new("minted_tokens");
