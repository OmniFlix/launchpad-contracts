#[cfg(not(feature = "library"))]
use cosmwasm_std::{Deps, DepsMut, Env, MessageInfo, Response};
use cw_utils::maybe_addr;
use minter_types::config::Config;
use minter_types::token_details::{MigrationNftError, Token, TokenDetails};
use omniflix_minter_factory::msg::QueryMsg::Params as QueryFactoryParams;
use omniflix_minter_factory::msg::{
    CreateMinterMsg, CreateMinterMsgWithMigration, MigrationData, ParamsResponse,
};
use serde::de;

use crate::error::ContractError;
use crate::state::{
    AUTH_DETAILS, COLLECTION, CONFIG, MINTABLE_TOKENS, TOKEN_DETAILS, TOTAL_TOKENS_REMAINING,
    USER_MINTING_DETAILS,
};
use crate::utils::randomize_token_list;
use minter_types::types::AuthDetails;

// TODO RENAME THIS
pub fn instantiate_with_migration(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: CreateMinterMsgWithMigration,
) -> Result<Response, ContractError> {
    // Query factory params of instantiator
    let _factory_params: ParamsResponse = deps
        .querier
        .query_wasm_smart(info.sender.clone().into_string(), &QueryFactoryParams {})?;

    // Get the migration data
    let migration_data = msg.migration_data;
    // Validate the migration data
    validate_migration_data(migration_data.clone(), deps.as_ref())?;

    let mintable_tokens = migration_data.mintable_tokens;
    let remaining_tokens_count = mintable_tokens.len() as u32;
    let minted_count = migration_data.minted_count;
    let auth_details = msg.auth_details.clone();
    let config = msg.config.clone();
    // Check if the total minted tokens are valid
    if remaining_tokens_count + minted_count != config.num_tokens.unwrap() {
        return Err(ContractError::InvalidMigrationMintedCount {});
    }
    let user_data = migration_data.users_data;
    // Check if collection details are valid
    let collection_details = msg.collection_details.clone();
    collection_details.check_integrity()?;

    // Create a empty token details
    // Wont be used for minting
    let token_details = TokenDetails::default();
    // Collect admin and payment collector
    let admin = deps.api.addr_validate(&auth_details.admin.into_string())?;
    let payment_collector = deps
        .api
        .addr_validate(&auth_details.payment_collector.into_string())?;

    // Create tokens with index
    let tokens = mintable_tokens
        .iter()
        .enumerate()
        .map(|(index, token)| (index as u32, token.clone()))
        .collect::<Vec<(u32, Token)>>();
    // Save the tokens
    let randomized_tokens =
        randomize_token_list(tokens.clone(), mintable_tokens.len() as u32, env.clone())?;
    randomized_tokens.iter().for_each(|(index, token)| {
        MINTABLE_TOKENS.save(deps.storage, *index, &token).unwrap();
    });
    TOTAL_TOKENS_REMAINING.save(deps.storage, &remaining_tokens_count)?;
    CONFIG.save(deps.storage, &config)?;
    COLLECTION.save(deps.storage, &collection_details)?;
    AUTH_DETAILS.save(
        deps.storage,
        &AuthDetails {
            admin,
            payment_collector,
        },
    )?;

    // Save user minting details
    user_data.iter().for_each(|(address, user_data)| {
        USER_MINTING_DETAILS
            .save(deps.storage, address.clone(), &user_data)
            .unwrap();
    });
    let res = Response::new().add_attribute("action", "instantiate_with_migration");
    Ok(res)
}

fn validate_migration_data(migration_data: MigrationData, deps: Deps) -> Result<(), ContractError> {
    // Check length of the migration data
    if migration_data.users_data.len() > 20_000 {
        return Err(ContractError::MigrationDataTooLarge {});
    }
    let mut user_addresses = vec![];
    let mintable_tokens = migration_data.mintable_tokens;
    let minted_count = migration_data.minted_count;
    let mut minted_tokens_sum = 0;

    // Check user data
    for user_data in migration_data.users_data {
        // Check if the user address is valid
        let _ = deps.api.addr_validate(&user_data.0.clone().into_string())?;
        // Check if any duplicate addresses are present
        if user_addresses.contains(&user_data.0.clone()) {
            return Err(ContractError::DuplicateUserAddress {});
        }
        user_addresses.push(user_data.0.clone());

        // Check if the user minted tokens are valid
        for token in user_data.1.minted_tokens {
            // Check if the token id is valid
            if token.migration_nft_data.is_none() {
                return Err(ContractError::MigrationNftError(
                    MigrationNftError::InvalidTokenMigrationData {},
                ));
            }
            token.migration_nft_data.unwrap().check_integrity()?;
            // Count every minted token
            // Should be equal to the total minted count provided by creator
            minted_tokens_sum += 1;
        }
    }

    // Check if the total minted tokens are valid
    if minted_tokens_sum != minted_count {
        return Err(ContractError::InvalidMigrationMintedCount {});
    }
    // Check tokens
    for token in mintable_tokens {
        // Check if the token id is valid
        if token.migration_nft_data.is_none() {
            return Err(ContractError::MigrationNftError(
                MigrationNftError::InvalidTokenMigrationData {},
            ));
        }
        token.migration_nft_data.unwrap().check_integrity()?;
    }
    Ok(())
}
