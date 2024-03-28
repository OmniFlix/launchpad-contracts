use std::env;
use std::str::FromStr;

use crate::msg::{ExecuteMsg, MinterExtensionQueryMsg};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Addr, Binary, Coin, CosmosMsg, Decimal, Deps, DepsMut, Env, MessageInfo, Order,
    Response, StdResult, Uint128, WasmMsg,
};
use cw_utils::{may_pay, maybe_addr, must_pay, nonpayable};
use minter_types::utils::{
    check_collection_creation_fee, generate_create_denom_msg, generate_minter_mint_message,
    generate_update_denom_msg, update_collection_details,
};
use omniflix_minter_factory::msg::QueryMsg::Params as QueryFactoryParams;
use omniflix_minter_factory::msg::{CreateMinterMsg, ParamsResponse};
use omniflix_round_whitelist::msg::ExecuteMsg::PrivateMint;
use omniflix_std::types::cosmos::auth;
use whitelist_types::{
    check_if_address_is_member, check_if_whitelist_is_active, check_whitelist_price,
};

use crate::error::ContractError;
use crate::state::{
    AUTH_DETAILS, COLLECTION, CONFIG, MINTABLE_TOKENS, TOKEN_DETAILS, TOTAL_TOKENS_REMAINING,
    USER_MINTING_DETAILS,
};
use crate::utils::{
    collect_mintable_tokens, generate_tokens, randomize_token_list, return_random_token,
};
use minter_types::msg::QueryMsg as BaseMinterQueryMsg;
use minter_types::types::{
    AuthDetails, CollectionDetails, Config, MigrationData, MigrationNftError, Token, TokenDetails,
    UserDetails,
};
use pauser::PauseState;

use cw2::set_contract_version;
use omniflix_std::types::omniflix::onft::v1beta1::{MsgPurgeDenom, WeightedAddress};
// TODO RENAME THIS
pub fn instantiate_with_migration(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: CreateMinterMsg,
) -> Result<Response, ContractError> {
    // We check if the migration data is present
    if msg.migration_data.is_none() {
        return Err(ContractError::MigrationDataNotFound {});
    }

    // Query factory params of instantiator
    let _factory_params: ParamsResponse = deps
        .querier
        .query_wasm_smart(info.sender.clone().into_string(), &QueryFactoryParams {})?;

    // Get the migration data
    let migration_data = msg.migration_data.unwrap();
    // Validate the migration data
    validate_migration_data(migration_data.clone(), deps.as_ref())?;
    let mintable_tokens = migration_data.mintable_tokens;
    let minted_count = migration_data.minted_count;

    let init = msg.init.clone();
    let collection_details = msg.collection_details.clone();
    collection_details.check_integrity()?;

    let token_details = TokenDetails::default();

    let config = Config {
        start_time: init.start_time,
        end_time: init.end_time,
        per_address_limit: init.per_address_limit,
        mint_price: init.mint_price,
        whitelist_address: maybe_addr(deps.api, init.whitelist_address)?,
        num_tokens: Some(init.num_tokens),
    };
    // Collect admin and payment collector
    let admin = deps.api.addr_validate(&init.admin)?;
    let payment_collector = match init.payment_collector {
        Some(payment_collector) => {
            let payment_collector_address = deps.api.addr_validate(&payment_collector)?;
            payment_collector_address
        }
        None => admin.clone(),
    };
    for i in 0..mintable_tokens.len() {
        let token = mintable_tokens[i].clone();
        MINTABLE_TOKENS.save(deps.storage, i as u32, &token)?;
    }

    Ok(Response::default())
}

fn validate_migration_data(migration_data: MigrationData, deps: Deps) -> Result<(), ContractError> {
    // Check length of the migration data
    if migration_data.users_data.len() > 20_000 {
        return Err(ContractError::MigrationDataTooLarge {});
    }
    let mut user_addresses = vec![];
    let tokens = migration_data.mintable_tokens;
    let minted_count = migration_data.minted_count;
    let mut minted_tokens_sum = 0;

    // Check user data
    for user_data in migration_data.users_data {
        // Check if the user address is valid
        let _ = deps.api.addr_validate(&user_data.0.into_string())?;
        // Check if any duplicate addresses are present
        if user_addresses.contains(&user_data.0) {
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
    for token in tokens {
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
