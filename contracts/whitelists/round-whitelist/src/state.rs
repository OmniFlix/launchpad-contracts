use std::error::Error;

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Coin, Deps, Order, StdError, StdResult, Storage, Timestamp};
use cw_storage_plus::{Item, Map};

use crate::error::ContractError;
use types::Round;

#[cw_serde]
pub struct Config {
    pub admin: Addr,
}
pub trait RoundMethods {
    fn is_active(&self, current_time: Timestamp) -> bool;
    fn is_member(&self, address: &Addr) -> bool;
    fn has_started(&self, current_time: Timestamp) -> bool;
    fn has_ended(&self, current_time: Timestamp) -> bool;
    fn start_time(&self) -> Timestamp;
    fn end_time(&self) -> Timestamp;
    fn round_per_address_limit(&self) -> u32;
    fn members(
        &self,
        start_after: Option<String>,
        limit: Option<u32>,
    ) -> Result<Vec<String>, ContractError>;
    fn mint_price(&self) -> Coin;
    fn check_integrity(&self, deps: Deps, now: Timestamp) -> Result<(), ContractError>;
}
impl RoundMethods for Round {
    fn is_active(&self, current_time: Timestamp) -> bool {
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
    fn is_member(&self, address: &Addr) -> bool {
        match self {
            Round::WhitelistAddresses { addresses, .. } => addresses.contains(address),
            Round::WhitelistCollection { .. } => false,
        }
    }

    fn has_started(&self, current_time: Timestamp) -> bool {
        match self {
            Round::WhitelistAddresses { start_time, .. } => current_time >= *start_time,
            Round::WhitelistCollection { start_time, .. } => current_time >= *start_time,
        }
    }

    fn has_ended(&self, current_time: Timestamp) -> bool {
        match self {
            Round::WhitelistAddresses { end_time, .. } => current_time >= *end_time,
            Round::WhitelistCollection { end_time, .. } => current_time >= *end_time,
        }
    }

    fn start_time(&self) -> Timestamp {
        match self {
            Round::WhitelistAddresses { start_time, .. } => *start_time,
            Round::WhitelistCollection { start_time, .. } => *start_time,
        }
    }

    fn end_time(&self) -> Timestamp {
        match self {
            Round::WhitelistAddresses { end_time, .. } => *end_time,
            Round::WhitelistCollection { end_time, .. } => *end_time,
        }
    }
    fn round_per_address_limit(&self) -> u32 {
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
    fn members(
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
    fn mint_price(&self) -> Coin {
        match self {
            Round::WhitelistAddresses { mint_price, .. } => mint_price.clone(),
            Round::WhitelistCollection { mint_price, .. } => mint_price.clone(),
        }
    }

    fn check_integrity(&self, deps: Deps, now: Timestamp) -> Result<(), ContractError> {
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

// Refactor RoundMints to use a map instead of a vector
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
            .filter_map(|result| result.ok().map(|(_, v)| v))
            .find(|round| round.is_active(current_time))
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
                return Err(ContractError::RoundsOverlaped {});
            }
        }
        Ok(())
    }
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const ROUND_MINTS: Map<Addr, RoundMints> = Map::new("round_mints");
pub const ROUNDS_KEY: &str = "rounds";

#[cfg(test)]
mod tests {
    use crate::error;

    use super::*;
    use cosmwasm_std::testing::mock_dependencies;
    use cosmwasm_std::{coin, Addr, CosmosMsg, Empty, MessageInfo, Response, SubMsg, WasmMsg};

    #[test]
    fn test_rounds_save() {
        let mut deps = mock_dependencies();
        let rounds = Rounds::new("rounds");
        let round = Round::WhitelistAddresses {
            addresses: vec![Addr::unchecked("addr1"), Addr::unchecked("addr2")],
            start_time: Timestamp::from_seconds(1000),
            end_time: Timestamp::from_seconds(2000),
            mint_price: coin(100, "flix"),
            round_per_address_limit: 1,
        };
        let round2 = Round::WhitelistAddresses {
            addresses: vec![Addr::unchecked("addr1"), Addr::unchecked("addr2")],
            start_time: Timestamp::from_seconds(3000),
            end_time: Timestamp::from_seconds(4000),
            mint_price: coin(100, "atom"),
            round_per_address_limit: 1,
        };
        let round1_index = rounds.save(&mut deps.storage, &round).unwrap();
        assert_eq!(round1_index, 1);
        assert_eq!(rounds.load(&deps.storage, round1_index).unwrap(), round);

        let round2_index = rounds.save(&mut deps.storage, &round2).unwrap();
        assert_eq!(round2_index, 2);
        assert_eq!(rounds.load(&deps.storage, round2_index).unwrap(), round2);

        let loadled_rounds = rounds.load_all_rounds(&deps.storage).unwrap();
        assert_eq!(loadled_rounds.len(), 2);
        assert_eq!(loadled_rounds[0], round);
        assert_eq!(loadled_rounds[1], round2);
    }

    #[test]
    fn test_rounds_remove() {
        let mut deps = mock_dependencies();
        let rounds = Rounds::new("rounds");
        let round = Round::WhitelistAddresses {
            addresses: vec![Addr::unchecked("addr1"), Addr::unchecked("addr2")],
            start_time: Timestamp::from_seconds(1000),
            end_time: Timestamp::from_seconds(2000),
            mint_price: coin(100, "flix"),
            round_per_address_limit: 1,
        };
        let round2 = Round::WhitelistAddresses {
            addresses: vec![Addr::unchecked("addr1"), Addr::unchecked("addr2")],
            start_time: Timestamp::from_seconds(3000),
            end_time: Timestamp::from_seconds(4000),
            mint_price: coin(100, "atom"),
            round_per_address_limit: 1,
        };
        let round1_index = rounds.save(&mut deps.storage, &round).unwrap();
        let round2_index = rounds.save(&mut deps.storage, &round2).unwrap();

        rounds.remove(&mut deps.storage, round1_index).unwrap();
        let loadled_rounds = rounds.load_all_rounds(&deps.storage).unwrap();
        assert_eq!(loadled_rounds.len(), 1);
        assert_eq!(loadled_rounds[0], round2);
    }

    #[test]
    fn test_rounds_load_active_round() {
        let mut deps = mock_dependencies();
        let rounds = Rounds::new("rounds");
        let round = Round::WhitelistAddresses {
            addresses: vec![Addr::unchecked("addr1"), Addr::unchecked("addr2")],
            start_time: Timestamp::from_seconds(1000),
            end_time: Timestamp::from_seconds(2000),
            mint_price: coin(100, "flix"),
            round_per_address_limit: 1,
        };
        let round2 = Round::WhitelistAddresses {
            addresses: vec![Addr::unchecked("addr1"), Addr::unchecked("addr2")],
            start_time: Timestamp::from_seconds(3000),
            end_time: Timestamp::from_seconds(4000),
            mint_price: coin(100, "atom"),
            round_per_address_limit: 1,
        };
        // Try to load active round when no round is saved
        let active_round = rounds.load_active_round(&deps.storage, Timestamp::from_seconds(1500));
        assert_eq!(active_round, None);

        let round1_index = rounds.save(&mut deps.storage, &round).unwrap();
        let round2_index = rounds.save(&mut deps.storage, &round2).unwrap();
        let loaded_rounds = rounds.load_all_rounds(&deps.storage).unwrap();
        assert_eq!(loaded_rounds.len(), 2);

        let active_round = rounds
            .load_active_round(&deps.storage, Timestamp::from_seconds(1500))
            .unwrap();
        assert_eq!(active_round, round);

        let active_round = rounds
            .load_active_round(&deps.storage, Timestamp::from_seconds(3500))
            .unwrap();
        assert_eq!(active_round, round2);

        let active_round = rounds.load_active_round(&deps.storage, Timestamp::from_seconds(5000));
        assert_eq!(active_round, None);
        let active_round = rounds.load_active_round(&deps.storage, Timestamp::from_seconds(0));
        assert_eq!(active_round, None);

        // Check load active round with overlapping rounds
        let round3 = Round::WhitelistAddresses {
            addresses: vec![Addr::unchecked("addr1"), Addr::unchecked("addr2")],
            start_time: Timestamp::from_seconds(1500),
            end_time: Timestamp::from_seconds(2500),
            mint_price: coin(100, "atom"),
            round_per_address_limit: 1,
        };

        let round3_index = rounds.save(&mut deps.storage, &round3).unwrap();
        let active_round = rounds
            .load_active_round(&deps.storage, Timestamp::from_seconds(1600))
            .unwrap();
        // We wont let that happen but if it does we will return the first round that is active
        assert_eq!(active_round, round);
    }

    #[test]
    fn test_rounds_check_round_overlaps() {
        let mut deps = mock_dependencies();
        let rounds = Rounds::new("rounds");
        let round = Round::WhitelistAddresses {
            addresses: vec![Addr::unchecked("addr1"), Addr::unchecked("addr2")],
            start_time: Timestamp::from_seconds(1000),
            end_time: Timestamp::from_seconds(2000),
            mint_price: coin(100, "flix"),
            round_per_address_limit: 1,
        };
        let round2 = Round::WhitelistAddresses {
            addresses: vec![Addr::unchecked("addr1"), Addr::unchecked("addr2")],
            start_time: Timestamp::from_seconds(3000),
            end_time: Timestamp::from_seconds(4000),
            mint_price: coin(100, "atom"),
            round_per_address_limit: 1,
        };
        let round3 = Round::WhitelistAddresses {
            addresses: vec![Addr::unchecked("addr1"), Addr::unchecked("addr2")],
            start_time: Timestamp::from_seconds(1500),
            end_time: Timestamp::from_seconds(2500),
            mint_price: coin(100, "atom"),
            round_per_address_limit: 1,
        };
        let round1_index = rounds.save(&mut deps.storage, &round).unwrap();
        let round2_index = rounds.save(&mut deps.storage, &round2).unwrap();
        // No overlap so unwrap should not fail
        rounds.check_round_overlaps(&deps.storage, None).unwrap();
        let error = rounds
            .check_round_overlaps(&deps.storage, Some(round3.clone()))
            .unwrap_err();
        assert_eq!(error, ContractError::RoundsOverlaped {});
    }
    #[test]
    fn test_try_mint() {
        let mut deps = mock_dependencies();
        let round = Round::WhitelistAddresses {
            addresses: vec![Addr::unchecked("addr1"), Addr::unchecked("addr2")],
            start_time: Timestamp::from_seconds(1000),
            end_time: Timestamp::from_seconds(2000),
            mint_price: coin(100, "flix"),
            round_per_address_limit: 1,
        };
        let round2 = Round::WhitelistAddresses {
            addresses: vec![Addr::unchecked("addr1"), Addr::unchecked("addr2")],
            start_time: Timestamp::from_seconds(3000),
            end_time: Timestamp::from_seconds(4000),
            mint_price: coin(100, "atom"),
            round_per_address_limit: 1,
        };
        let mut round_mints = RoundMints::new();
        // First mint should pass
        round_mints.try_mint(round.clone()).unwrap();
        // Second mint should fail
        let error = round_mints.try_mint(round.clone()).unwrap_err();
        assert_eq!(error, ContractError::RoundReachedMintLimit {});
        // First mint of round2 should pass
        round_mints.try_mint(round2.clone()).unwrap();
        // Second mint of round2 should fail
        let error = round_mints.try_mint(round2.clone()).unwrap_err();
        assert_eq!(error, ContractError::RoundReachedMintLimit {});
    }
}
