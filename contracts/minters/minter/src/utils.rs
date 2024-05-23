use std::convert::TryInto;

use cosmwasm_std::{Env, Order, StdError, Storage};
use minter_types::token_details::Token;
use rand_core::{RngCore, SeedableRng};
use rand_xoshiro::Xoshiro128PlusPlus;
use sha2::{Digest, Sha256};
use shuffle::{fy::FisherYates, shuffler::Shuffler};

use crate::state::MINTABLE_TOKENS;

pub fn randomize_token_list(
    tokens: Vec<(u32, Token)>,
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
    // Shuffle tokens
    let mut shuffler = FisherYates::default();
    shuffler
        .shuffle(&mut raw_tokens, &mut rng)
        .map_err(StdError::generic_err)?;
    // Iterate over tokens
    let mut randomized_tokens: Vec<(u32, Token)> = Vec::new();
    let mut key: u32 = 1;
    for token in raw_tokens {
        randomized_tokens.push((key, token));
        key += 1;
    }
    Ok(randomized_tokens)
}

pub fn return_random_token_index(
    num_of_tokens: u32,
    env: Env,
    storage: &dyn Storage,
) -> Result<u32, StdError> {
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

    let divider = 50.min(num_of_tokens);
    // We should limit the amount of tokens we skip to prevent gas exhaustion
    let token_skip_amount = r % divider;

    let random_token_position: u32 = MINTABLE_TOKENS
        .keys(storage, None, None, Order::Ascending)
        .skip(token_skip_amount as usize)
        .take(1)
        .collect::<Result<Vec<u32>, StdError>>()?[0];

    Ok(random_token_position)
}

pub fn generate_tokens(num_of_tokens: u32) -> Vec<(u32, Token)> {
    let tokens: Vec<(u32, Token)> = (1..=num_of_tokens)
        .map(|x| {
            (
                x,
                Token {
                    token_id: x.to_string(),
                },
            )
        })
        .collect();
    tokens
}
#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::{
        testing::{mock_dependencies, mock_env},
        Timestamp, TransactionInfo,
    };

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
    fn test_return_random_token() {
        // Generate vector of 1000 elements from 1 to 1000
        let mut deps = mock_dependencies();
        let total_tokens = 1000;
        let mut env = mock_env();
        env.block.height = 652678625765;
        env.block.time = Timestamp::from_nanos(782787);
        env.transaction = Some(TransactionInfo { index: 121474982 });

        let tokens = generate_tokens(total_tokens);

        // Save tokens
        for token in tokens.clone() {
            MINTABLE_TOKENS
                .save(deps.as_mut().storage, token.0, &token.1)
                .unwrap();
        }

        let random_token_index =
            return_random_token_index(total_tokens, env, deps.as_ref().storage).unwrap();

        // Random index should be between 1 and num of tokens
        assert!(random_token_index >= 1 && random_token_index <= total_tokens);
    }
    #[test]
    fn redo_random_pick() {
        let mut deps = mock_dependencies();
        // Regenerate token list
        let mut env = mock_env();
        env.block.height = 100_000;
        env.block.time = Timestamp::from_nanos(200_000);
        env.transaction = Some(TransactionInfo { index: 400_000 + 1 });
        let mut total_tokens = 1000;
        let tokens = generate_tokens(total_tokens);

        let randomized_list =
            randomize_token_list(tokens.clone(), total_tokens, env.clone()).unwrap();

        // save tokens
        for token in randomized_list.clone() {
            MINTABLE_TOKENS
                .save(deps.as_mut().storage, token.0, &token.1)
                .unwrap();
        }

        // Pick a token from the list let's say it's index 5
        let picked_token = &randomized_list[4];

        // Count how many times it takes to pick the token
        let mut count = 0;

        loop {
            let random_token_index =
                return_random_token_index(total_tokens, env.clone(), deps.as_ref().storage)
                    .unwrap()
                    .clone();
            let random_token = MINTABLE_TOKENS
                .load(deps.as_ref().storage, random_token_index)
                .unwrap();

            count += 1;

            if random_token == picked_token.1 {
                break;
            } else {
                MINTABLE_TOKENS.remove(deps.as_mut().storage, random_token_index);
                total_tokens -= 1;
            }
        }

        println!("Final Count: {}", count);
        println!(
            "Remaining token count: {}",
            MINTABLE_TOKENS
                .range(deps.as_ref().storage, None, None, Order::Ascending)
                .count() as u32
                - 1 as u32
        );
        //Spoiler allert - it takes 270 times to pick the token
        //Add 1 to tx index and it takes 123 times
    }
}
