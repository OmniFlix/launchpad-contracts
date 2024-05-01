use cosmwasm_std::{Coin, CosmosMsg, DepsMut, Env, MessageInfo, Response};
use cw_utils::{maybe_addr, must_pay};
use minter_types::config::Config;
use minter_types::utils::{check_collection_creation_fee, generate_create_denom_msg};
use omniflix_minter_factory::msg::QueryMsg::Params as QueryFactoryParams;
use omniflix_minter_factory::msg::{CreateMinterMsg, ParamsResponse};
use std::env;
use whitelist_types::check_if_whitelist_is_active;

use crate::error::ContractError;
use crate::state::{
    AUTH_DETAILS, COLLECTION, CONFIG, MINTABLE_TOKENS, TOKEN_DETAILS, TOTAL_TOKENS_REMAINING,
};
use crate::utils::{generate_tokens, randomize_token_list};
use pauser::PauseState;

use cw2::set_contract_version;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:omniflix-minter";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn default_instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: CreateMinterMsg,
) -> Result<Response, ContractError> {
    // Set contract version
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    // Query factory params of instantiator
    let _factory_params: ParamsResponse = deps
        .querier
        .query_wasm_smart(info.sender.clone().into_string(), &QueryFactoryParams {})?;

    // Check collection creation fee
    let collection_creation_fee: Coin = check_collection_creation_fee(deps.as_ref().querier)?;

    // Clone message init for further use
    let init = msg.init.clone();

    // Validate token details
    let token_details = msg
        .token_details
        .clone()
        .ok_or(ContractError::InvalidTokenDetails {})?;

    let collection_details = msg.collection_details.clone();

    // Check if whitelist is active
    if let Some(whitelist_address) = init.whitelist_address.clone() {
        let is_active = check_if_whitelist_is_active(
            &deps.api.addr_validate(&whitelist_address)?,
            deps.as_ref(),
        )?;
        if is_active {
            return Err(ContractError::WhitelistAlreadyActive {});
        }
    }

    // Initialize config
    let config = Config {
        per_address_limit: init.per_address_limit,
        start_time: init.start_time,
        mint_price: init.mint_price,
        whitelist_address: maybe_addr(deps.api, init.whitelist_address.clone())?,
        end_time: init.end_time,
        num_tokens: Some(init.num_tokens),
    };
    // Check config integrity
    config.check_integrity(env.block.time)?;
    // Check token details integrity
    token_details.check_integrity()?;

    // Validate payment amount
    let amount = must_pay(&info, &collection_creation_fee.denom)?;
    if amount != collection_creation_fee.amount {
        return Err(ContractError::InvalidCreationFee {
            expected: vec![collection_creation_fee.clone()],
            sent: info.funds,
        });
    }

    // Validate authorization details
    let auth_details = msg.auth_details.clone();
    auth_details.validate(&deps.as_ref())?;

    // Save configuration and authorization details
    CONFIG.save(deps.storage, &config)?;
    AUTH_DETAILS.save(deps.storage, &auth_details)?;
    COLLECTION.save(deps.storage, &collection_details)?;
    TOKEN_DETAILS.save(deps.storage, &token_details)?;

    // Generate and save tokens
    let tokens = generate_tokens(init.num_tokens);
    let randomized_list = randomize_token_list(tokens.clone(), init.num_tokens, env.clone())?;
    for token in randomized_list {
        MINTABLE_TOKENS.save(deps.storage, token.0, &token.1)?;
    }

    // Save total tokens
    TOTAL_TOKENS_REMAINING.save(deps.storage, &init.num_tokens)?;

    // Initialize pause state and set admin as pauser
    let pause_state = PauseState::new()?;
    pause_state.set_pausers(
        deps.storage,
        info.sender.clone(),
        vec![auth_details.admin.clone()],
    )?;

    // Generate create denom message
    let collection_creation_msg: CosmosMsg = generate_create_denom_msg(
        &collection_details,
        env.contract.address,
        collection_creation_fee,
        auth_details.payment_collector,
    )?
    .into();
    let res = Response::new()
        .add_message(collection_creation_msg)
        .add_attribute("action", "instantiate");

    Ok(res)
}
