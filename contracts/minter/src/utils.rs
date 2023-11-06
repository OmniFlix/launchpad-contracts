use std::convert::TryInto;

use cosmwasm_std::{
    from_binary, from_json, Addr, Binary, Deps, DepsMut, Env, Order, StdError, Timestamp,
};
use omniflix_std::types::omniflix::onft::v1beta1::{OnftQuerier, QueryCollectionResponse};
use rand_core::{RngCore, SeedableRng};
use rand_xoshiro::Xoshiro128PlusPlus;
use serde::de::value;
use sha2::{Digest, Sha256};
use shuffle::{fy::FisherYates, shuffler::Shuffler};

use crate::{
    error::ContractError,
    state::{Config, Round, Token, CONFIG, MINTED_TOKENS},
};
use types::whitelist::Config as WhitelistConfig;
use types::whitelist::{
    HasMemberResponse, IsActiveResponse, PerAddressLimitResponse, WhitelistQueryMsgs,
};

pub fn randomize_token_list(
    mut tokens: Vec<(u32, Token)>,
    total_tokens: u32,
    env: Env,
) -> Result<Vec<(u32, Token)>, StdError> {
    let tx_index: u32 = if let Some(tx) = env.transaction {
        tx.index
    } else {
        0
    };
    // Collect tokens
    let mut raw_tokens: Vec<Token> = tokens.into_iter().map(|x| x.1).collect();
    let sha256 = Sha256::digest(format!(
        "{}{}{}{}",
        env.block.height, env.block.time, tx_index, total_tokens
    ));
    let randomness: [u8; 16] = sha256.to_vec()[0..16].try_into().unwrap();
    let mut rng = Xoshiro128PlusPlus::from_seed(randomness);

    let mut shuffler = FisherYates::default();
    shuffler
        .shuffle(&mut raw_tokens, &mut rng)
        .map_err(StdError::generic_err)?;
    // Iterate over tokens
    let mut randomized_tokens: Vec<(u32, Token)> = Vec::new();
    // TODO: is it ok to reset all keys for every randomization?
    let mut key: u32 = 1;
    for token in raw_tokens {
        randomized_tokens.push((key, token));
        key += 1;
    }
    Ok(randomized_tokens)
}

pub fn return_random_token_id(
    token_list: &Vec<(u32, Token)>,
    env: Env,
) -> Result<(u32, Token), StdError> {
    // We are expecting mintable tokens and corresponding keys an an vector
    let tokens = token_list;

    // Generate random token id
    let tx_index: u32 = if let Some(tx) = env.transaction {
        tx.index
    } else {
        0
    };
    let sha256 = Sha256::digest(format!(
        "{}{}{}",
        env.block.height, env.block.time, tx_index
    ));
    let randomness: [u8; 16] = sha256.to_vec()[0..16].try_into().unwrap();

    let mut rng = Xoshiro128PlusPlus::from_seed(randomness);

    let r = rng.next_u32();

    let is_ascending = r % 2 == 0;

    let lenght = tokens.clone().len() as u32;
    let random_index = r % lenght;

    match is_ascending {
        true => {
            let random_token = &tokens.clone()[random_index as usize];
            Ok(random_token.clone())
        }
        false => {
            let random_token = &tokens.clone()[lenght as usize - random_index as usize - 1];
            Ok(random_token.clone())
        }
    }
}

pub fn check_round_overlaps(
    now: Timestamp,
    rounds: Vec<(u32, Round)>,
    public_start_time: Timestamp,
) -> Result<(), ContractError> {
    let mut rounds = rounds;

    // add public as a round
    rounds.push((
        u32::MAX,
        Round::WhitelistAddress {
            address: Addr::unchecked("public"),
            start_time: Some(public_start_time),
            // There is no public mint end time we generate 100 day after start time to be safe
            end_time: Some(public_start_time.plus_days(100)),
            mint_price: Default::default(),
            round_limit: Default::default(),
        },
    ));
    // Sort rounds by start time
    rounds.sort_by(|a, b| a.1.start_time().cmp(&b.1.start_time()));
    // Check for overlaps
    for (i, round) in rounds.iter().enumerate() {
        if i == rounds.len() - 1 {
            break;
        }
        let next_round = &rounds[i + 1];
        if round.1.end_time() > next_round.1.start_time() {
            return Err(ContractError::RoundsOverlaped {
                round: round.1.clone(),
            });
        }
    }
    // Check for overlaps with now none of them should be started
    for round in rounds {
        if round.1.start_time() < now {
            return Err(ContractError::RoundAlreadyStarted {});
        }
    }
    Ok(())
}
pub fn return_updated_round(deps: &DepsMut, round: Round) -> Result<Round, ContractError> {
    match round {
        Round::WhitelistAddress {
            address,
            start_time,
            end_time,
            mint_price,
            round_limit,
        } => {
            let whitelist_config: WhitelistConfig = deps
                .querier
                .query_wasm_smart(address.clone(), &WhitelistQueryMsgs::Config {})?;
            let round = Round::WhitelistAddress {
                address,
                start_time: Some(whitelist_config.start_time),
                end_time: Some(whitelist_config.end_time),
                mint_price: whitelist_config.mint_price.amount,
                round_limit: whitelist_config.per_address_limit,
            };
            Ok(round)
        }
        Round::WhitelistCollection {
            collection_id,
            start_time,
            end_time,
            mint_price,
            round_limit,
        } => {
            let round = Round::WhitelistCollection {
                collection_id: collection_id.clone(),
                start_time,
                end_time,
                mint_price,
                round_limit,
            };
            Ok(round)
        }
    }
}

pub fn check_if_whitelisted(
    member: String,
    round: Round,
    deps: Deps,
) -> Result<bool, ContractError> {
    match round {
        Round::WhitelistAddress {
            address,
            start_time,
            end_time,
            mint_price,
            round_limit,
        } => {
            let has_member_response: HasMemberResponse = deps.querier.query_wasm_smart(
                address,
                &WhitelistQueryMsgs::HasMember {
                    member: member.clone(),
                },
            )?;
            if has_member_response.has_member {
                return Ok(true);
            }
        }
        Round::WhitelistCollection {
            collection_id,
            start_time,
            end_time,
            mint_price,
            round_limit,
        } => {
            let onft_querier = OnftQuerier::new(&deps.querier);
            // TODO: Check if there is better way
            let owner_amount = onft_querier.supply(collection_id, member)?;
            if owner_amount.amount > 0 {
                return Ok(true);
            }
        }
    }

    Ok(false)
}

pub fn find_active_round(
    now: Timestamp,
    rounds: Vec<(u32, Round)>,
) -> Result<(u32, Round), ContractError> {
    let mut rounds = rounds;
    // Sort rounds by start time
    rounds.sort_by(|a, b| a.1.start_time().cmp(&b.1.start_time()));
    // Find active round
    for round in rounds {
        if round.1.start_time() <= now && round.1.end_time() >= now {
            return Ok(round);
        }
    }
    Err(ContractError::RoundEnded {})
}
#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::{testing::mock_env, TransactionInfo, Uint128};

    #[test]
    fn test_randomize_token_list() {
        // Generate vector of 1000 elements from 1 to 1000
        let tokens: Vec<(u32, Token)> = (1..=1000)
            .map(|x| {
                (
                    x,
                    Token {
                        token_id: x.to_string(),
                    },
                )
            })
            .collect();
        let total_tokens = 1000;
        let mut env = mock_env();
        env.block.height = 657625347635765;
        env.block.time = Timestamp::from_nanos(782784568767866);
        env.transaction = Some(TransactionInfo { index: 12147492 });

        let randomized_list = randomize_token_list(tokens.clone(), total_tokens, env).unwrap();

        assert_ne!(randomized_list, tokens);
    }

    #[test]
    fn test_return_random_token_id() {
        // Generate vector of 1000 elements from 1 to 1000
        let tokens: Vec<(u32, Token)> = (1..=1000)
            .map(|x| {
                (
                    x,
                    Token {
                        token_id: x.to_string(),
                    },
                )
            })
            .collect();
        let total_tokens = 1000;
        let mut env = mock_env();
        env.block.height = 652678625765;
        env.block.time = Timestamp::from_nanos(782787);
        env.transaction = Some(TransactionInfo { index: 121474982 });

        let randomized_list =
            randomize_token_list(tokens.clone(), total_tokens, env.clone()).unwrap();
        let random_token = return_random_token_id(&randomized_list.clone(), env).unwrap();
        // This random token should have a key and a token. The key and token should be between 1 and 1000
        assert!(random_token.0 >= 1 && random_token.0 <= 1000);
        assert!(
            random_token.1.token_id.parse::<u32>().unwrap() >= 1
                && random_token.1.token_id.parse::<u32>().unwrap() <= 1000
        );
        // Regenerate token list
        let mut env = mock_env();
        env.block.height = 100_000;
        env.block.time = Timestamp::from_nanos(200_000);
        env.transaction = Some(TransactionInfo { index: 400_000 });

        let mut randomized_list =
            randomize_token_list(tokens.clone(), total_tokens, env.clone()).unwrap();

        // Pick a token from the list let's say it's 5
        let picked_token = &randomized_list[4];

        // Count how many times it takes to pick the token
        let mut count = 0;
        let mut modified_list = randomized_list.clone(); // Create a mutable copy

        loop {
            let random_token = return_random_token_id(&modified_list, env.clone())
                .unwrap()
                .clone();
            count += 1;

            if random_token == *picked_token {
                break;
            } else {
                // Remove token from the mutable copy of the list
                modified_list.retain(|x| x != &random_token);
            }
        }

        println!("Final Count: {}", count);
        println!("Modified List Count: {:?}", modified_list.clone().len());
        // Spoiler allert - it takes 896 times to pick the token
        // Add 1 to tx index and it takes 123 times
    }
    #[test]
    fn test_no_overlap() {
        // Three non-overlapping rounds
        let round1 = Round::WhitelistAddress {
            address: Addr::unchecked("A"),
            start_time: Some(Timestamp::from_seconds(2)),
            end_time: Some(Timestamp::from_seconds(5)),
            mint_price: Uint128::new(100),
            round_limit: 1,
        };
        let round2 = Round::WhitelistAddress {
            address: Addr::unchecked("C"),
            start_time: Some(Timestamp::from_seconds(5)),
            end_time: Some(Timestamp::from_seconds(7)),
            mint_price: Uint128::new(100),
            round_limit: 1,
        };
        let round3 = Round::WhitelistAddress {
            address: Addr::unchecked("E"),
            start_time: Some(Timestamp::from_seconds(7)),
            end_time: Some(Timestamp::from_seconds(9)),
            mint_price: Uint128::new(100),
            round_limit: 1,
        };

        let now = Timestamp::from_seconds(0);
        let public_start_time = Timestamp::from_seconds(10);
        let public_end_time = Timestamp::from_seconds(12);

        let rounds: Vec<(u32, Round)> = vec![(1, round1), (2, round2), (3, round3)];

        // Check for no overlaps
        let result = check_round_overlaps(now, rounds, public_start_time);
        assert!(result.is_ok());
    }

    #[test]
    fn test_overlap_between_rounds() {
        // Three rounds with overlaps between round 1 and round 2
        let round1 = Round::WhitelistAddress {
            address: Addr::unchecked("A"),
            start_time: Some(Timestamp::from_seconds(0)),
            end_time: Some(Timestamp::from_seconds(3)),
            mint_price: Uint128::new(100),
            round_limit: 1,
        };
        let round2 = Round::WhitelistAddress {
            address: Addr::unchecked("C"),
            start_time: Some(Timestamp::from_seconds(2)),
            end_time: Some(Timestamp::from_seconds(4)),
            mint_price: Uint128::new(100),
            round_limit: 1,
        };
        let round3 = Round::WhitelistAddress {
            address: Addr::unchecked("E"),
            start_time: Some(Timestamp::from_seconds(5)),
            end_time: Some(Timestamp::from_seconds(7)),
            mint_price: Uint128::new(100),
            round_limit: 1,
        };

        let now = Timestamp::from_seconds(0);
        let public_start_time = Timestamp::from_seconds(0);
        let public_end_time = Timestamp::from_seconds(8);

        let rounds: Vec<(u32, Round)> = vec![(1, round1), (2, round2), (3, round3)];

        // Check for overlap between rounds 1 and 2
        let result = check_round_overlaps(now, rounds, public_start_time);
        assert!(result.is_err());
    }

    #[test]
    fn test_overlap_with_public_time() {
        // Three rounds with overlaps between round 1 and public time
        let round1 = Round::WhitelistAddress {
            address: Addr::unchecked("A"),
            start_time: Some(Timestamp::from_seconds(0)),
            end_time: Some(Timestamp::from_seconds(3)),
            mint_price: Uint128::new(100),
            round_limit: 1,
        };
        let round2 = Round::WhitelistAddress {
            address: Addr::unchecked("C"),
            start_time: Some(Timestamp::from_seconds(4)),
            end_time: Some(Timestamp::from_seconds(5)),
            mint_price: Uint128::new(100),
            round_limit: 1,
        };
        let round3 = Round::WhitelistAddress {
            address: Addr::unchecked("E"),
            start_time: Some(Timestamp::from_seconds(5)),
            end_time: Some(Timestamp::from_seconds(7)),
            mint_price: Uint128::new(100),
            round_limit: 1,
        };

        let now = Timestamp::from_seconds(0);
        let public_start_time = Timestamp::from_seconds(0);
        let public_end_time = Timestamp::from_seconds(9);

        let rounds: Vec<(u32, Round)> = vec![(1, round1), (2, round2), (3, round3)];

        // Check for overlap between round 1 and public time
        let result = check_round_overlaps(now, rounds, public_start_time);
        assert!(result.is_err());
    }

    #[test]
    fn test_find_active_round() {
        // Generete 3 rounds
        let round1 = Round::WhitelistAddress {
            address: Addr::unchecked("A"),
            start_time: Some(Timestamp::from_seconds(0)),
            end_time: Some(Timestamp::from_seconds(3)),
            mint_price: Uint128::new(100),
            round_limit: 1,
        };
        let round2 = Round::WhitelistAddress {
            address: Addr::unchecked("C"),
            start_time: Some(Timestamp::from_seconds(4)),
            end_time: Some(Timestamp::from_seconds(5)),
            mint_price: Uint128::new(100),
            round_limit: 1,
        };
        let round3 = Round::WhitelistAddress {
            address: Addr::unchecked("E"),
            start_time: Some(Timestamp::from_seconds(5)),
            end_time: Some(Timestamp::from_seconds(7)),
            mint_price: Uint128::new(100),
            round_limit: 1,
        };

        let now = Timestamp::from_seconds(0);

        let rounds: Vec<(u32, Round)> = vec![(1, round1), (2, round2), (3, round3)];
        let result = find_active_round(now, rounds.clone()).unwrap();
        assert_eq!(result.0, 1);

        let now = Timestamp::from_seconds(9);
        let result = find_active_round(now, rounds).unwrap_err();
        assert_eq!(result, ContractError::RoundEnded {});
    }
}
