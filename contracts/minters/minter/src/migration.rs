#[cfg(not(feature = "library"))]
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};
use minter_types::token_details::Token;
use omniflix_minter_factory::msg::QueryMsg::Params as QueryFactoryParams;
use omniflix_minter_factory::msg::{CreateMinterMsgWithMigration, ParamsResponse};

use crate::error::ContractError;
use crate::state::{
    AUTH_DETAILS, COLLECTION, CONFIG, MINTABLE_TOKENS, TOKEN_DETAILS, TOTAL_TOKENS_REMAINING,
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

    // Check if the total minted tokens are valid
    if remaining_tokens_count + minted_count != config.num_tokens.unwrap() {
        return Err(ContractError::InvalidMigrationMintedCount {});
    }

    // Check if collection details are valid
    collection_details.check_integrity()?;

    // Collect admin and payment collector
    let admin = deps.api.addr_validate(&auth_details.admin.into_string())?;
    let payment_collector = deps
        .api
        .addr_validate(&auth_details.payment_collector.into_string())?;

    // Enumerate the tokens
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
    TOKEN_DETAILS.save(deps.storage, &token_details)?;
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
    let res = Response::new().add_attribute("action", "instantiate_with_migration");

    Ok(res)
}
