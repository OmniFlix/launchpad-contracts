use std::convert::TryInto;

use cosmwasm_std::{Env, StdError};
use minter_types::Token;
use rand_core::{RngCore, SeedableRng};
use rand_xoshiro::Xoshiro128PlusPlus;
use sha2::{Digest, Sha256};
use shuffle::{fy::FisherYates, shuffler::Shuffler};

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

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::{testing::mock_env, Timestamp, TransactionInfo, Uint128};

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
}
