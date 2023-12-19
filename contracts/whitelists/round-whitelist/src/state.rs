use std::error::Error;

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Coin, Deps, Order, StdError, StdResult, Storage, Timestamp};
use cw_storage_plus::{Item, Map};

use crate::error::ContractError;

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
#[cw_serde]
pub struct Config {
    pub admin: Addr,
}

impl Round {
    pub fn is_active(&self, current_time: Timestamp) -> bool {
        match self {
            Round::WhitelistAddresses {
                start_time,
                end_time,
                ..
            } => current_time >= *start_time && current_time <= *end_time,
            Round::WhitelistCollection {
                start_time,
                end_time,
                ..
            } => current_time >= *start_time && current_time <= *end_time,
        }
    }

    pub fn has_started(&self, current_time: Timestamp) -> bool {
        match self {
            Round::WhitelistAddresses { start_time, .. } => current_time >= *start_time,
            Round::WhitelistCollection { start_time, .. } => current_time >= *start_time,
        }
    }

    pub fn has_ended(&self, current_time: Timestamp) -> bool {
        match self {
            Round::WhitelistAddresses { end_time, .. } => current_time >= *end_time,
            Round::WhitelistCollection { end_time, .. } => current_time >= *end_time,
        }
    }

    pub fn start_time(&self) -> Timestamp {
        match self {
            Round::WhitelistAddresses { start_time, .. } => *start_time,
            Round::WhitelistCollection { start_time, .. } => *start_time,
        }
    }

    pub fn end_time(&self) -> Timestamp {
        match self {
            Round::WhitelistAddresses { end_time, .. } => *end_time,
            Round::WhitelistCollection { end_time, .. } => *end_time,
        }
    }
    pub fn round_per_address_limit(&self) -> u32 {
        match self {
            Round::WhitelistAddresses {
                round_per_address_limit,
                ..
            } => *round_per_address_limit,
            Round::WhitelistCollection {
                round_per_address_limit,
                ..
            } => *round_per_address_limit,
        }
    }
    pub fn members(
        &self,
        start_after: Option<String>,
        limit: Option<u32>,
    ) -> Result<Vec<String>, ContractError> {
        match self {
            Round::WhitelistAddresses { addresses, .. } => {
                let mut members: Vec<String> = addresses.iter().map(|x| x.to_string()).collect();
                let start_after = start_after.unwrap_or_default();
                let start_index = members
                    .iter()
                    .position(|x| x.as_str() == start_after.as_str())
                    .unwrap_or_default();
                let end_index = match limit {
                    Some(limit) => start_index + limit as usize,
                    None => members.len(),
                };
                Ok(members[start_index..end_index].to_vec())
            }
            Round::WhitelistCollection { .. } => Err(ContractError::InvalidRoundType {
                expected: "WhitelistAddresses".to_string(),
                actual: "WhitelistCollection".to_string(),
            }),
        }
    }
    pub fn mint_price(&self) -> Coin {
        match self {
            Round::WhitelistAddresses { mint_price, .. } => mint_price.clone(),
            Round::WhitelistCollection { mint_price, .. } => mint_price.clone(),
        }
    }

    pub fn check_integrity(&self, deps: Deps, now: Timestamp) -> Result<(), ContractError> {
        match self {
            Round::WhitelistAddresses {
                addresses,
                start_time,
                end_time,
                mint_price,
                round_per_address_limit,
            } => {
                if addresses.is_empty() {
                    return Err(ContractError::EmptyAddressList {});
                }
                if now >= *start_time {
                    return Err(ContractError::InvalidStartTime {});
                }
                if *start_time >= *end_time {
                    return Err(ContractError::InvalidStartTime {});
                }
                if *round_per_address_limit == 0 {
                    return Err(ContractError::InvalidPerAddressLimit {});
                }
                for address in addresses {
                    deps.api.addr_validate(address.as_str())?;
                }
            }
            Round::WhitelistCollection {
                collection_id,
                start_time,
                end_time,
                mint_price,
                round_per_address_limit,
            } => {
                // TODO: Validate collection id by Querying the collection
                if collection_id.is_empty() {
                    return Err(ContractError::InvalidMemberLimit {});
                }
                if now >= *start_time {
                    return Err(ContractError::InvalidStartTime {});
                }
                if *start_time >= *end_time {
                    return Err(ContractError::InvalidStartTime {});
                }
                if *round_per_address_limit == 0 {
                    return Err(ContractError::InvalidPerAddressLimit {});
                }
            }
        }
        Ok(())
    }
}
pub type MintCount = u32;
#[cw_serde]
pub struct RoundMints {
    pub rounds: Vec<(Round, MintCount)>,
}

impl RoundMints {
    pub fn new() -> Self {
        RoundMints { rounds: Vec::new() }
    }
    pub fn try_mint(&mut self, active_round: Round) -> Result<(), ContractError> {
        let mut mint_round = self
            .rounds
            .iter_mut()
            .find(|(round, _)| round == &active_round);
        if mint_round.is_none() {
            let mut mint_round = (active_round, 0);
            mint_round.1 += 1;
            if mint_round.0.round_per_address_limit() < mint_round.1 {
                return Err(ContractError::RoundReachedMintLimit {});
            };
            self.rounds.push(mint_round);
        } else {
            let mint_round = mint_round.unwrap();
            mint_round.1 += 1;
            if active_round.round_per_address_limit() < mint_round.1 {
                return Err(ContractError::RoundReachedMintLimit {});
            };
        }
        Ok(())
    }
}

//pub const ROUNDS: Map<u32, Round> = Map::new("mintable_tokens");

pub struct Rounds<'a>(Map<'a, u32, Round>);
impl<'a> Rounds<'a> {
    pub const fn new(storage_key: &'a str) -> Self {
        Rounds(Map::new(storage_key))
    }

    pub fn save(&self, store: &mut dyn Storage, round: &Round) -> StdResult<u32> {
        let last_id = self
            .0
            .range(store, None, None, Order::Descending)
            .next()
            .transpose()?
            .map(|(id, _)| id)
            .unwrap_or(0);

        self.0.save(store, last_id + 1, round)?;

        Ok(last_id + 1)
    }

    pub fn load(&self, store: &dyn Storage, id: u32) -> StdResult<Round> {
        Ok(self
            .0
            .may_load(store, id)?
            .ok_or_else(|| StdError::generic_err("Round not found"))?)
    }
    pub fn remove(&self, store: &mut dyn Storage, id: u32) -> StdResult<()> {
        Ok(self.0.remove(store, id))
    }

    pub fn load_active_round(&self, store: &dyn Storage, current_time: Timestamp) -> Option<Round> {
        self.0
            .range(store, None, None, Order::Ascending)
            .map(|result| result.map(|(_, v)| v))
            .flatten()
            .next()
    }

    pub fn load_all_rounds(&self, store: &dyn Storage) -> StdResult<Vec<Round>> {
        self.0
            .range(store, None, None, Order::Ascending)
            .map(|x| x.map(|(_, v)| v))
            .collect()
    }
    pub fn check_round_overlaps(
        &self,
        store: &dyn Storage,
        round: Option<Round>,
    ) -> Result<(), ContractError> {
        let mut rounds = self.load_all_rounds(store)?;
        if let Some(round) = round {
            rounds.push(round);
        }
        rounds.sort_by_key(|round| round.start_time());

        for i in 0..rounds.len() - 1 {
            let current_round = &rounds[i];
            let next_round = &rounds[i + 1];

            if current_round.end_time() > next_round.start_time() {
                return Err(ContractError::InvalidRoundTime {
                    round: current_round.clone(),
                });
            }
        }
        Ok(())
    }
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const ROUND_MINTS: Map<Addr, RoundMints> = Map::new("round_mints");
pub const ROUNDS_KEY: &str = "rounds";
