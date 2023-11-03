use std::u32;

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Decimal, Timestamp, Uint128};
use cw_storage_plus::{Item, Map};

use crate::{error::ContractError, msg::CollectionDetails};

#[cw_serde]
pub struct Config {
    pub per_address_limit: u32,
    pub payment_collector: Addr,
    pub mint_denom: String,
    pub start_time: Timestamp,
    pub mint_price: Uint128,
    pub royalty_ratio: Decimal,
    pub creator: Addr,
}

#[cw_serde]
pub struct Token {
    pub token_id: String,
}
#[cw_serde]
pub enum Round {
    WhitelistAddress {
        address: Addr,
        start_time: Option<Timestamp>,
        end_time: Option<Timestamp>,
        mint_price: Uint128,
        round_limit: u32,
    },
    WhitelistCollection {
        collection_id: String,
        start_time: Timestamp,
        end_time: Timestamp,
        mint_price: Uint128,
        round_limit: u32,
    },
}
impl Round {
    pub fn start_time(&self) -> Timestamp {
        match self {
            Round::WhitelistAddress { start_time, .. } => start_time.unwrap(),
            Round::WhitelistCollection { start_time, .. } => *start_time,
        }
    }
    pub fn end_time(&self) -> Timestamp {
        match self {
            Round::WhitelistAddress { end_time, .. } => end_time.unwrap(),
            Round::WhitelistCollection { end_time, .. } => *end_time,
        }
    }
    pub fn mint_price(&self) -> Uint128 {
        match self {
            Round::WhitelistAddress { .. } => Uint128::zero(),
            Round::WhitelistCollection { mint_price, .. } => *mint_price,
        }
    }
    pub fn round_limit(&self) -> u32 {
        match self {
            Round::WhitelistAddress { round_limit, .. } => *round_limit,
            Round::WhitelistCollection { round_limit, .. } => *round_limit,
        }
    }
    pub fn return_whitelist_address(&self) -> Option<Addr> {
        match self {
            Round::WhitelistAddress { address, .. } => Some(address.clone()),
            Round::WhitelistCollection { .. } => None,
        }
    }
    pub fn update_params(
        &mut self,
        start_time: Option<Timestamp>,
        end_time: Option<Timestamp>,
        mint_price: Option<Uint128>,
        round_limit: Option<u32>,
    ) -> Result<(), ContractError> {
        match self {
            Round::WhitelistAddress {
                start_time: ref mut s,
                end_time: ref mut e,
                mint_price: ref mut m,
                round_limit: ref mut r,
                ..
            } => {
                if let Some(start_time) = start_time {
                    *s = Some(start_time);
                }
                if let Some(end_time) = end_time {
                    *e = Some(end_time);
                }
                if let Some(mint_price) = mint_price {
                    *m = mint_price;
                }
                if let Some(round_limit) = round_limit {
                    *r = round_limit;
                }
            }
            Round::WhitelistCollection {
                start_time: ref mut s,
                end_time: ref mut e,
                mint_price: ref mut m,
                round_limit: ref mut r,
                ..
            } => {
                if let Some(start_time) = start_time {
                    *s = start_time;
                }
                if let Some(end_time) = end_time {
                    *e = end_time;
                }
                if let Some(mint_price) = mint_price {
                    *m = mint_price;
                }
                if let Some(round_limit) = round_limit {
                    *r = round_limit;
                }
            }
        }
        Ok(())
    }
    pub fn return_round_type(&self) -> String {
        match self {
            Round::WhitelistAddress { .. } => "address".to_string(),
            Round::WhitelistCollection { .. } => "collection".to_string(),
        }
    }
}

#[cw_serde]
pub struct UserDetails {
    pub minted_tokens: Vec<Token>,
    pub total_minted_count: u32,
    pub rounds_mints: Vec<MintCountInRound>,
}

#[cw_serde]
pub struct MintCountInRound {
    pub round_index: u32,
    pub count: u32,
}

impl UserDetails {
    pub fn new() -> Self {
        UserDetails {
            minted_tokens: Vec::new(),
            total_minted_count: 0,
            rounds_mints: Vec::new(),
        }
    }
    pub fn add_minted_token(
        &mut self,
        per_address_limit: u32,
        round_limit: Option<u32>,
        token: Token,
        // If no round index is provided, its a public mint.
        round_index: Option<u32>,
    ) -> Result<(), ContractError> {
        if let Some(round_index) = round_index {
            // Find the round in rounds_mints and modify it if it exists
            if let Some(round) = self
                .rounds_mints
                .iter_mut()
                .find(|r| r.round_index == round_index)
            {
                round.count += 1;
                self.total_minted_count += 1;

                if self.total_minted_count > per_address_limit {
                    return Err(ContractError::AddressReachedMintLimit {});
                }

                // Check if a round_limit is provided and validate the count
                if let Some(limit) = round_limit {
                    if round.count > limit {
                        return Err(ContractError::RoundReachedMintLimit {});
                    }
                }
            } else {
                // If the round doesn't exist, create it and add it to rounds_mints.
                self.total_minted_count += 1;
                if self.total_minted_count > per_address_limit {
                    return Err(ContractError::AddressReachedMintLimit {});
                }
                self.rounds_mints.push(MintCountInRound {
                    round_index,
                    count: 1,
                });
            }
            self.minted_tokens.push(token);
        } else {
            self.total_minted_count += 1;
            if self.total_minted_count > per_address_limit {
                return Err(ContractError::AddressReachedMintLimit {});
            }
            self.minted_tokens.push(token.clone());
        }
        Ok(())
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
// Rounds
pub const ROUNDS: Map<u32, Round> = Map::new("rounds");
