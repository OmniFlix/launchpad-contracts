use crate::round::RoundMethods;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Api, Order, StdResult, Storage, Timestamp};
use cw_storage_plus::{Item, Map};

use crate::error::ContractError;
use whitelist_types::Round;

pub const CONFIG: Item<Config> = Item::new("config");
pub const ROUNDMEMBERS: Map<(Vec<u8>, Vec<u8>), bool> = Map::new("round_members");
pub const ROUNDS_KEY: &str = "rounds";
pub const USERMINTDETAILS_KEY: &str = "user_mint_details";

#[cw_serde]
pub struct Config {
    pub admin: Addr,
}

pub type MintCount = u32;
pub type RoundIndex = u8;

#[cw_serde]
pub struct MintDetails {
    pub rounds: Vec<(RoundIndex, MintCount)>,
}
pub type MinterAddress = Addr;
pub type UserAddress = Addr;
pub struct UserMintDetails<'a>(Map<'a, (UserAddress, MinterAddress), MintDetails>);
impl<'a> UserMintDetails<'a> {
    pub const fn new(storage_key: &'a str) -> Self {
        UserMintDetails(Map::new(storage_key))
    }

    pub fn mint_for_user(
        &self,
        store: &mut dyn Storage,
        user_address: &UserAddress,
        minter_address: &MinterAddress,
        round_index: &u8,
        round: &Round,
    ) -> Result<(), ContractError> {
        // Check if user exist
        let mut user_mint_details = self
            .0
            .may_load(store, (user_address.clone(), minter_address.clone()))?
            .unwrap_or(MintDetails { rounds: Vec::new() });

        // Find the index of the round inside the user_mint_details
        let user_mint_index = user_mint_details
            .rounds
            .iter()
            .position(|(found_round_index, _)| found_round_index == round_index);

        if let Some(index) = user_mint_index {
            // Increment the mint count for the existing round
            user_mint_details.rounds[index].1 += 1;

            // Check if the updated mint count exceeds the round_per_address_limit
            if user_mint_details.rounds[index].1 > round.round_per_address_limit {
                return Err(ContractError::RoundReachedMintLimit {});
            }
        } else {
            // Round not found, add a new entry for the round
            user_mint_details.rounds.push((*round_index, 1));
        }

        // Save the updated user_mint_details
        self.0.save(
            store,
            (user_address.clone(), minter_address.clone()),
            &user_mint_details,
        )?;
        Ok(())
    }
}

pub struct Rounds<'a>(Map<'a, RoundIndex, Round>);
impl<'a> Rounds<'a> {
    pub const fn new(storage_key: &'a str) -> Self {
        Rounds(Map::new(storage_key))
    }

    pub fn save(&self, store: &mut dyn Storage, round: &Round) -> StdResult<u8> {
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

    pub fn update(&self, store: &mut dyn Storage, id: u8, round: &Round) -> StdResult<()> {
        self.0.save(store, id, round)?;
        Ok(())
    }
    pub fn last_id(&self, store: &dyn Storage) -> StdResult<u8> {
        let last_id = self
            .0
            .range(store, None, None, Order::Descending)
            .next()
            .transpose()?
            .map(|(id, _)| id)
            .unwrap_or(0);

        Ok(last_id)
    }

    pub fn load(&self, store: &dyn Storage, id: u8) -> Result<Round, ContractError> {
        self.0
            .may_load(store, id)?
            .ok_or(ContractError::RoundNotFound {})
    }
    pub fn remove(&self, store: &mut dyn Storage, id: u8) -> StdResult<()> {
        self.0.remove(store, id);
        Ok(())
    }
    pub fn load_active_round(
        &self,
        store: &dyn Storage,
        current_time: Timestamp,
    ) -> Option<(u8, Round)> {
        self.0
            .range(store, None, None, Order::Ascending)
            .filter_map(|result| result.ok())
            // We are assuming that there are no overlapping rounds in storage
            // So the first round that is active is the active round
            .find(|(_, round)| round.is_active(current_time))
    }

    pub fn load_all_rounds(&self, store: &dyn Storage) -> StdResult<Vec<(u8, Round)>> {
        self.0.range(store, None, None, Order::Ascending).collect()
    }

    pub fn check_round_overlaps(
        &self,
        store: &dyn Storage,
        // It has option to check if the provided rounds overlap with the rounds in storage
        round: Option<Vec<Round>>,
    ) -> Result<(), ContractError> {
        let last_index = self.last_id(store)?;
        let mut rounds = self.load_all_rounds(store)?;

        // Put indexes of the provided rounds
        // Its not needed to start at last index + 1 because we are only checking for overlaps
        if let Some(round) = round {
            for r in round {
                rounds.push((last_index + 1, r));
            }
        }

        // Sort the rounds by start time
        rounds.sort_by(|(_, a), (_, b)| a.start_time.cmp(&b.start_time));

        // Check if any round overlaps
        for i in 0..rounds.len() - 1 {
            let current_round = &rounds[i];
            let next_round = &rounds[i + 1];

            if current_round.1.end_time > next_round.1.start_time {
                return Err(ContractError::RoundsOverlapped {});
            }
        }
        // [(1, Round { start_time: Timestamp(Uint64(2000)), end_time: Timestamp(Uint64(3000)), mint_price: Coin { 1000000 "diffirent_denom" }, round_per_address_limit: 1 }),
        // (2, Round { start_time: Timestamp(Uint64(4000)), end_time: Timestamp(Uint64(5000)), mint_price: Coin { 1000000 "uflix" }, round_per_address_limit: 1 }),
        //(3, Round { start_time: Timestamp(Uint64(4001)), end_time: Timestamp(Uint64(24334243)), mint_price: Coin { 1000000 "uflix" }, round_per_address_limit: 1 })]

        Ok(())
    }
}

pub fn save_members(
    store: &mut dyn Storage,
    api: &dyn Api,
    round_index: u8,
    members: &Vec<String>,
) -> Result<(), ContractError> {
    const MAX_MEMBERS: usize = 5000;
    if members.len() > MAX_MEMBERS {
        return Err(ContractError::WhitelistMemberLimitExceeded {});
    }
    if members.is_empty() {
        return Err(ContractError::EmptyAddressList {});
    }
    for member in members {
        let validated = api.addr_validate(member.as_str())?;
        let address_str = validated.as_str();
        let round_index_str = round_index.to_string();
        ROUNDMEMBERS.save(
            store,
            (
                round_index_str.as_bytes().to_vec(),
                address_str.as_bytes().to_vec(),
            ),
            &true,
        )?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {

    use super::*;
    use cosmwasm_std::testing::mock_dependencies;
    use cosmwasm_std::{coin, Addr};

    #[test]
    fn test_rounds_save() {
        let mut deps = mock_dependencies();
        let rounds = Rounds::new("rounds");
        let round = Round {
            start_time: Timestamp::from_seconds(1000),
            end_time: Timestamp::from_seconds(2000),
            mint_price: coin(100, "flix"),
            round_per_address_limit: 1,
        };
        let round2 = Round {
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
        assert_eq!(loadled_rounds[0].1, round);
        assert_eq!(loadled_rounds[1].1, round2);
    }

    #[test]
    fn test_rounds_remove() {
        let mut deps = mock_dependencies();
        let rounds = Rounds::new("rounds");
        let round = Round {
            start_time: Timestamp::from_seconds(1000),
            end_time: Timestamp::from_seconds(2000),
            mint_price: coin(100, "flix"),
            round_per_address_limit: 1,
        };
        let round2 = Round {
            start_time: Timestamp::from_seconds(3000),
            end_time: Timestamp::from_seconds(4000),
            mint_price: coin(100, "atom"),
            round_per_address_limit: 1,
        };
        let round1_index = rounds.save(&mut deps.storage, &round).unwrap();
        let _round2_index = rounds.save(&mut deps.storage, &round2).unwrap();

        rounds.remove(&mut deps.storage, round1_index).unwrap();
        let loadled_rounds = rounds.load_all_rounds(&deps.storage).unwrap();
        // Index 1 should be empty
        let _loaded_round_1 = rounds.load(&deps.storage, round1_index).unwrap_err();
        // Try saving a new round
        let round3 = Round {
            start_time: Timestamp::from_seconds(5000),
            end_time: Timestamp::from_seconds(6000),
            mint_price: coin(100, "atom"),
            round_per_address_limit: 1,
        };
        let round3_index = rounds.save(&mut deps.storage, &round3).unwrap();
        assert_eq!(round3_index, 3);
        assert_eq!(loadled_rounds.len(), 1);
        assert_eq!(loadled_rounds[0].1, round2);
    }

    #[test]
    fn test_rounds_load_active_round() {
        let mut deps = mock_dependencies();
        let rounds = Rounds::new("rounds");
        let round = Round {
            start_time: Timestamp::from_seconds(1000),
            end_time: Timestamp::from_seconds(2000),
            mint_price: coin(100, "flix"),
            round_per_address_limit: 1,
        };
        let round2 = Round {
            start_time: Timestamp::from_seconds(3000),
            end_time: Timestamp::from_seconds(4000),
            mint_price: coin(100, "atom"),
            round_per_address_limit: 1,
        };
        // Try to load active round when no round is saved
        let active_round = rounds.load_active_round(&deps.storage, Timestamp::from_seconds(1500));
        assert_eq!(active_round, None);

        let _round1_index = rounds.save(&mut deps.storage, &round).unwrap();
        let _round2_index = rounds.save(&mut deps.storage, &round2).unwrap();
        let loaded_rounds = rounds.load_all_rounds(&deps.storage).unwrap();
        assert_eq!(loaded_rounds.len(), 2);

        let active_round = rounds
            .load_active_round(&deps.storage, Timestamp::from_seconds(1500))
            .unwrap();
        assert_eq!(active_round.1, round);

        let active_round = rounds
            .load_active_round(&deps.storage, Timestamp::from_seconds(3500))
            .unwrap();
        assert_eq!(active_round.1, round2);

        let active_round = rounds.load_active_round(&deps.storage, Timestamp::from_seconds(5000));
        assert_eq!(active_round, None);
        let active_round = rounds.load_active_round(&deps.storage, Timestamp::from_seconds(0));
        assert_eq!(active_round, None);

        // Check load active round with overlapping rounds
        let round3 = Round {
            start_time: Timestamp::from_seconds(1500),
            end_time: Timestamp::from_seconds(2500),
            mint_price: coin(100, "atom"),
            round_per_address_limit: 1,
        };

        let _round3_index = rounds.save(&mut deps.storage, &round3).unwrap();
        let active_round = rounds
            .load_active_round(&deps.storage, Timestamp::from_seconds(1600))
            .unwrap();
        // We wont let that happen but if it does we will return the first round that is active
        assert_eq!(active_round.1, round);
    }

    #[test]
    fn test_rounds_check_round_overlaps() {
        let mut deps = mock_dependencies();
        let rounds = Rounds::new("rounds");
        let round = Round {
            start_time: Timestamp::from_seconds(1000),
            end_time: Timestamp::from_seconds(2000),
            mint_price: coin(100, "flix"),
            round_per_address_limit: 1,
        };
        let round2 = Round {
            start_time: Timestamp::from_seconds(3000),
            end_time: Timestamp::from_seconds(4000),
            mint_price: coin(100, "atom"),
            round_per_address_limit: 1,
        };
        let round3 = Round {
            start_time: Timestamp::from_seconds(1500),
            end_time: Timestamp::from_seconds(2500),
            mint_price: coin(100, "atom"),
            round_per_address_limit: 1,
        };
        let _round1_index = rounds.save(&mut deps.storage, &round).unwrap();
        let _round2_index = rounds.save(&mut deps.storage, &round2).unwrap();
        // No overlap so unwrap should not fail
        rounds.check_round_overlaps(&deps.storage, None).unwrap();
        let error = rounds
            .check_round_overlaps(&deps.storage, Some([round3.clone()].to_vec()))
            .unwrap_err();
        assert_eq!(error, ContractError::RoundsOverlapped {});
    }
    #[test]
    fn test_try_mint() {
        let mut deps = mock_dependencies();
        let user_details = UserMintDetails::new("user_mint_details");

        let round_1 = Round {
            start_time: Timestamp::from_seconds(1000),
            end_time: Timestamp::from_seconds(2000),
            mint_price: coin(100, "flix"),
            round_per_address_limit: 1,
        };

        let _round_2 = Round {
            start_time: Timestamp::from_seconds(3000),
            end_time: Timestamp::from_seconds(4000),
            mint_price: coin(100, "atom"),
            round_per_address_limit: 1,
        };

        // Create a new user
        let user_address = Addr::unchecked("user1");
        let minter_address = Addr::unchecked("minter1");

        // Try to mint for a user
        user_details
            .mint_for_user(
                &mut deps.storage,
                &user_address,
                &minter_address,
                &1,
                &round_1,
            )
            .unwrap();
        // Check if the user_mint_details is saved
        let user_mint_details = user_details
            .0
            .load(
                &deps.storage,
                (user_address.clone(), minter_address.clone()),
            )
            .unwrap();
        assert_eq!(
            user_mint_details,
            MintDetails {
                rounds: vec![(1, 1)]
            }
        );

        // Try to mint for a user again
        let res = user_details
            .mint_for_user(
                &mut deps.storage,
                &user_address,
                &minter_address,
                &1,
                &round_1,
            )
            .unwrap_err();
        assert_eq!(res, ContractError::RoundReachedMintLimit {});
    }
}
