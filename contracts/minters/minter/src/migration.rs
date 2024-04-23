#[cfg(not(feature = "library"))]
use cosmwasm_std::{Deps, DepsMut, Env, MessageInfo, Response};
use cw_utils::maybe_addr;
use minter_types::collection_details;
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

    let CreateMinterMsgWithMigration {
        migration_data,
        config,
        collection_details,
        auth_details,
        token_details,
    } = msg;

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
