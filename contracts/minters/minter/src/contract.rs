use std::env;
use std::str::FromStr;

use crate::instantiation::default_instantiate;
use crate::migration::instantiate_with_migration;
use crate::msg::{ExecuteMsg, MinterExtensionQueryMsg};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Addr, Binary, Coin, CosmosMsg, Decimal, Deps, DepsMut, Env, MessageInfo, Order,
    Response, StdResult, Uint128, WasmMsg,
};
use cw_utils::{may_pay, nonpayable};
use minter_types::collection_details::CollectionDetails;
use minter_types::config::Config;
use minter_types::token_details::{Token, TokenDetails};
use minter_types::utils::{
    generate_minter_mint_message, generate_update_denom_msg, update_collection_details,
};

use omniflix_minter_factory::msg::CreateMinterMsgs;
use omniflix_round_whitelist::msg::ExecuteMsg::PrivateMint;
use whitelist_types::{
    check_if_address_is_member, check_if_whitelist_is_active, check_whitelist_price,
};

use crate::error::ContractError;
use crate::state::{
    AUTH_DETAILS, COLLECTION, CONFIG, MINTABLE_TOKENS, TOKEN_DETAILS, TOTAL_TOKENS_REMAINING,
    USER_MINTING_DETAILS,
};
use crate::utils::{collect_mintable_tokens, randomize_token_list, return_random_token};
use minter_types::msg::QueryMsg as BaseMinterQueryMsg;
use minter_types::types::{AuthDetails, UserDetails};
use pauser::PauseState;

use cw2::set_contract_version;
use omniflix_std::types::omniflix::onft::v1beta1::{MsgPurgeDenom, WeightedAddress};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:omniflix-minter";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn handle_instantiation(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: CreateMinterMsgs,
) -> Result<Response, ContractError> {
    // Set contract version
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    match msg {
        CreateMinterMsgs::CreateMinter { msg } => default_instantiate(deps, env, info, msg),
        CreateMinterMsgs::CreateMinterWithMigration { msg } => {
            instantiate_with_migration(deps, env, info, msg)
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Mint {} => execute_mint(deps, env, info),
        ExecuteMsg::MintAdmin {
            recipient,
            token_id,
        } => execute_mint_admin(deps, env, info, recipient, token_id),
        ExecuteMsg::BurnRemainingTokens {} => execute_burn_remaining_tokens(deps, env, info),
        ExecuteMsg::UpdateRoyaltyRatio { ratio } => {
            execute_update_royalty_ratio(deps, env, info, ratio)
        }
        ExecuteMsg::UpdateMintPrice { mint_price } => {
            execute_update_mint_price(deps, env, info, mint_price)
        }
        ExecuteMsg::RandomizeList {} => execute_randomize_list(deps, env, info),
        ExecuteMsg::UpdateWhitelistAddress { address } => {
            execute_update_whitelist_address(deps, env, info, address)
        }
        ExecuteMsg::UpdateAdmin { admin } => execute_update_admin(deps, env, info, admin),
        ExecuteMsg::UpdatePaymentCollector { payment_collector } => {
            execute_update_payment_collector(deps, env, info, payment_collector)
        }
        ExecuteMsg::Pause {} => execute_pause(deps, env, info),
        ExecuteMsg::Unpause {} => execute_unpause(deps, env, info),
        ExecuteMsg::SetPausers { pausers } => execute_set_pausers(deps, env, info, pausers),
        ExecuteMsg::UpdateRoyaltyReceivers { receivers } => {
            execute_update_royalty_receivers(deps, env, info, receivers)
        }
        ExecuteMsg::UpdateDenom {
            collection_name,
            description,
            preview_uri,
        } => execute_update_denom(deps, env, info, collection_name, description, preview_uri),
        ExecuteMsg::PurgeDenom {} => execute_purge_denom(deps, env, info),
    }
}

pub fn execute_mint(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    // Check if the contract is paused
    let pause_state = PauseState::new()?;
    pause_state.error_if_paused(deps.storage)?;

    // Load configuration and authorization details
    let config = CONFIG.load(deps.storage)?;
    let auth_details = AUTH_DETAILS.load(deps.storage)?;

    // Check remaining tokens
    let total_tokens_remaining = TOTAL_TOKENS_REMAINING.load(deps.storage)?;
    if total_tokens_remaining == 0 {
        return Err(ContractError::NoTokensLeftToMint {});
    }

    // Load user minting details or initialize with defaults
    let mut user_details = USER_MINTING_DETAILS
        .may_load(deps.storage, info.sender.clone())?
        .unwrap_or_default();

    // Load mint price
    let mut mint_price = config.mint_price;

    // Collect mintable tokens
    let mintable_tokens = collect_mintable_tokens(deps.as_ref().storage)?;

    // Check if public minting is started and if end time is passed
    let is_public = env.block.time >= config.start_time;
    if let Some(end_time) = config.end_time {
        if env.block.time > end_time {
            return Err(ContractError::PublicMintingEnded {});
        }
    }

    // Get a random token
    let random_token = return_random_token(&mintable_tokens, env.clone())?;

    let mut messages: Vec<CosmosMsg> = vec![];

    if !is_public {
        // Only for private minting

        // Check if any whitelist is present
        if let Some(whitelist_address) = config.whitelist_address {
            // Check if whitelist is active
            let is_active = check_if_whitelist_is_active(&whitelist_address, deps.as_ref())?;
            if !is_active {
                return Err(ContractError::WhitelistNotActive {});
            }

            // Check whitelist price
            let whitelist_price = check_whitelist_price(&whitelist_address, deps.as_ref())?;
            mint_price = whitelist_price;

            // Check if member is whitelisted
            let is_member = check_if_address_is_member(
                &info.sender.clone(),
                &whitelist_address,
                deps.as_ref(),
            )?;
            if !is_member {
                return Err(ContractError::AddressNotWhitelisted {});
            }

            // Execute private mint message
            messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: whitelist_address.into_string(),
                msg: to_json_binary(&PrivateMint {
                    collector: info.sender.clone().into_string(),
                })?,
                funds: vec![],
            }));
        } else {
            return Err(ContractError::MintingNotStarted {
                start_time: config.start_time,
                current_time: env.block.time,
            });
        };
    } else {
        // Only for public minting

        user_details.public_mint_count += 1;
        // Check if per address limit is reached
        if let Some(per_address_limit) = config.per_address_limit {
            if user_details.public_mint_count > per_address_limit {
                return Err(ContractError::AddressReachedMintLimit {});
            }
        }
    }

    // Increment total minted count
    user_details.total_minted_count += 1;

    // Add minted token to user details
    user_details.minted_tokens.push(random_token.1.clone());

    // Save user details
    USER_MINTING_DETAILS.save(deps.storage, info.sender.clone(), &user_details)?;

    // Check payment amount
    let amount = may_pay(&info, &mint_price.denom)?;
    // Exact amount must be paid
    if amount != mint_price.amount {
        return Err(ContractError::IncorrectPaymentAmount {
            expected: mint_price.amount,
            sent: amount,
        });
    }

    // Get payment collector address
    let payment_collector = auth_details.payment_collector;

    // Load collection and token details
    let collection = COLLECTION.load(deps.storage)?;
    let token_details = TOKEN_DETAILS.load(deps.storage)?;

    // Update storage
    MINTABLE_TOKENS.remove(deps.storage, random_token.0);
    TOTAL_TOKENS_REMAINING.update(deps.storage, |mut total_tokens| -> StdResult<_> {
        total_tokens -= 1;
        Ok(total_tokens)
    })?;

    let token_id = random_token.1.clone().token_id;

    // Generate mint message
    let mint_msg: CosmosMsg = generate_minter_mint_message(
        &collection,
        &token_details,
        token_id.clone(),
        env.contract.address,
        info.sender,
        random_token.1.clone(),
        auth_details.admin,
    )?
    .into();

    // Generate bank send message if payment amount is non-zero
    if !mint_price.amount.is_zero() {
        let bank_msg: CosmosMsg = CosmosMsg::Bank(cosmwasm_std::BankMsg::Send {
            to_address: payment_collector.into_string(),
            amount: vec![Coin {
                denom: mint_price.denom,
                amount: mint_price.amount,
            }],
        });
        messages.push(bank_msg);
    }

    // Add mint message to messages
    messages.push(mint_msg.clone());

    // Generate response
    let res = Response::new()
        .add_messages(messages)
        .add_attribute("action", "mint")
        .add_attribute("token_id", token_id.to_string())
        .add_attribute("collection_id", collection.id);

    Ok(res)
}

pub fn execute_mint_admin(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    recipient: String,
    token_id: Option<String>,
) -> Result<Response, ContractError> {
    // Error if paused
    let pause_state = PauseState::new()?;
    pause_state.error_if_paused(deps.storage)?;

    nonpayable(&info)?;
    let collection = COLLECTION.load(deps.storage)?;
    let auth_details = AUTH_DETAILS.load(deps.storage)?;

    // Verify sender is admin
    if info.sender != auth_details.admin {
        return Err(ContractError::Unauthorized {});
    }

    // Validate recipient address
    let recipient = deps.api.addr_validate(&recipient)?;

    // Collect mintable tokens
    let mintable_tokens = collect_mintable_tokens(deps.as_ref().storage)?;

    // Retrieve token to mint
    let token = match token_id {
        None => return_random_token(&mintable_tokens, env.clone())?,
        Some(token_id) => {
            // Find token by ID
            let token: Option<(u32, Token)> = mintable_tokens
                .iter()
                .find(|(_, token)| token.token_id == token_id)
                .map(|(key, token)| (*key, token.clone()));

            match token {
                None => return Err(ContractError::TokenIdNotMintable {}),
                Some(token) => token,
            }
        }
    };

    // Remove token from mintable tokens
    MINTABLE_TOKENS.remove(deps.storage, token.0);

    // Decrement total tokens remaining
    TOTAL_TOKENS_REMAINING.update(deps.storage, |mut total_tokens| -> StdResult<_> {
        total_tokens -= 1;
        Ok(total_tokens)
    })?;

    // Increment minted tokens count for recipient
    let mut user_details = USER_MINTING_DETAILS
        .may_load(deps.storage, recipient.clone())?
        .unwrap_or(UserDetails::default());
    // Update user details directly to override per address limit checks
    user_details.minted_tokens.push(token.1.clone());
    user_details.total_minted_count += 1;
    // Save user details
    USER_MINTING_DETAILS.save(deps.storage, recipient.clone(), &user_details)?;

    let token_id = token.1.clone().token_id;

    // Generate mint message
    let mint_msg: CosmosMsg = generate_minter_mint_message(
        &collection,
        &TOKEN_DETAILS.load(deps.storage)?,
        token_id.clone(),
        env.contract.address,
        recipient.clone(),
        token.1.clone(),
        auth_details.admin,
    )?
    .into();

    let res = Response::new()
        .add_message(mint_msg)
        .add_attribute("action", "mint")
        .add_attribute("token_id", token_id.to_string())
        .add_attribute("denom_id", collection.id);
    Ok(res)
}

pub fn execute_burn_remaining_tokens(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    // Check if sender is admin
    let auth_details = AUTH_DETAILS.load(deps.storage)?;
    if info.sender != auth_details.admin {
        return Err(ContractError::Unauthorized {});
    }
    // We technicaly cant burn tokens because they are not minted yet
    // But we can delete the mintable tokens map

    // Delete the mintable tokens map
    MINTABLE_TOKENS.clear(deps.storage);

    // Decrement the total tokens
    TOTAL_TOKENS_REMAINING.save(deps.storage, &0)?;

    let res = Response::new().add_attribute("action", "burn_remaining_tokens");
    Ok(res)
}

pub fn execute_update_royalty_ratio(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    ratio: String,
) -> Result<Response, ContractError> {
    // Check if sender is admin
    let auth_details = AUTH_DETAILS.load(deps.storage)?;
    if info.sender != auth_details.admin {
        return Err(ContractError::Unauthorized {});
    }

    let ratio = Decimal::from_str(&ratio)?;
    let mut token_details = TOKEN_DETAILS.load(deps.storage)?;
    token_details.royalty_ratio = ratio;
    token_details.check_integrity()?;
    TOKEN_DETAILS.save(deps.storage, &token_details)?;

    let res = Response::new()
        .add_attribute("action", "update_royalty_ratio")
        .add_attribute("ratio", ratio.to_string());
    Ok(res)
}

pub fn execute_update_admin(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    admin: String,
) -> Result<Response, ContractError> {
    // Check if sender is admin
    let mut auth_details = AUTH_DETAILS.load(deps.storage)?;
    if info.sender != auth_details.admin {
        return Err(ContractError::Unauthorized {});
    }
    let new_admin = deps.api.addr_validate(&admin)?;
    auth_details.admin = new_admin.clone();
    AUTH_DETAILS.save(deps.storage, &auth_details)?;

    let res = Response::new()
        .add_attribute("action", "update_admin")
        .add_attribute("admin", admin.to_string());
    Ok(res)
}

pub fn execute_update_payment_collector(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    payment_collector: String,
) -> Result<Response, ContractError> {
    // Check if sender is admin
    let mut auth_details = AUTH_DETAILS.load(deps.storage)?;
    if info.sender != auth_details.admin {
        return Err(ContractError::Unauthorized {});
    }
    let new_payment_collector = deps.api.addr_validate(&payment_collector)?;
    auth_details.payment_collector = new_payment_collector.clone();
    AUTH_DETAILS.save(deps.storage, &auth_details)?;

    let res = Response::new()
        .add_attribute("action", "update_payment_collector")
        .add_attribute("payment_collector", payment_collector.to_string());
    Ok(res)
}

pub fn execute_update_mint_price(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    mint_price: Coin,
) -> Result<Response, ContractError> {
    // Check if sender is admin
    let mut config = CONFIG.load(deps.storage)?;
    let auth_details = AUTH_DETAILS.load(deps.storage)?;
    if info.sender != auth_details.admin {
        return Err(ContractError::Unauthorized {});
    }
    // Check if mint price is valid
    if mint_price.amount == Uint128::new(0) {
        return Err(ContractError::InvalidMintPrice {});
    }
    config.mint_price = mint_price.clone();

    CONFIG.save(deps.storage, &config)?;

    let res = Response::new()
        .add_attribute("action", "update_mint_price")
        .add_attribute("mint_price_denom", mint_price.denom.to_string())
        .add_attribute("mint_price_amount", mint_price.amount.to_string());
    Ok(res)
}

pub fn execute_randomize_list(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    // Check if sender is admin
    let auth_details = AUTH_DETAILS.load(deps.storage)?;
    // This should be available for everyone but then this could be abused
    if info.sender != auth_details.admin {
        return Err(ContractError::Unauthorized {});
    }
    // Collect mintable tokens
    let mut mintable_tokens: Vec<(u32, Token)> = Vec::new();
    for item in MINTABLE_TOKENS.range(deps.storage, None, None, Order::Ascending) {
        let (key, value) = item?;

        // Add the (key, value) tuple to the vector
        mintable_tokens.push((key, value));
    }
    let tokens_remaining = TOTAL_TOKENS_REMAINING.load(deps.storage)?;
    let randomized_list = randomize_token_list(mintable_tokens, tokens_remaining, env)?;

    for token in randomized_list {
        MINTABLE_TOKENS.save(deps.storage, token.0, &token.1)?;
    }

    let res = Response::new().add_attribute("action", "randomize_list");
    Ok(res)
}

pub fn execute_update_whitelist_address(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    address: String,
) -> Result<Response, ContractError> {
    // Check if sender is admin
    let mut config = CONFIG.load(deps.storage)?;
    let auth_details = AUTH_DETAILS.load(deps.storage)?;
    if info.sender != auth_details.admin {
        return Err(ContractError::Unauthorized {});
    }
    let whitelist_address = config.whitelist_address.clone();
    // To update a whitelist address, we first check if one exists and is not active.
    // If it's active, we throw an error; the creator cannot update an active whitelist address or set an address that's already active.
    if whitelist_address.is_some() {
        let is_active = check_if_whitelist_is_active(&whitelist_address.unwrap(), deps.as_ref())?;
        if is_active {
            return Err(ContractError::WhitelistAlreadyActive {});
        }
    }
    let address = deps.api.addr_validate(&address)?;
    let is_active: bool = check_if_whitelist_is_active(&address, deps.as_ref())?;
    if is_active {
        return Err(ContractError::WhitelistAlreadyActive {});
    }
    config.whitelist_address = Some(address.clone());

    CONFIG.save(deps.storage, &config)?;

    let res = Response::new()
        .add_attribute("action", "update_whitelist_address")
        .add_attribute("address", address.to_string());
    Ok(res)
}

pub fn execute_pause(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let pause_state = PauseState::new()?;
    pause_state.pause(deps.storage, &info.sender)?;
    let res = Response::new().add_attribute("action", "pause");
    Ok(res)
}

pub fn execute_unpause(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    // Check if sender is admin
    let pause_state = PauseState::new()?;
    pause_state.unpause(deps.storage, &info.sender)?;
    let res = Response::new().add_attribute("action", "unpause");
    Ok(res)
}

pub fn execute_set_pausers(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    pausers: Vec<String>,
) -> Result<Response, ContractError> {
    let pause_state = PauseState::new()?;
    pause_state.set_pausers(
        deps.storage,
        info.sender,
        pausers
            .iter()
            .map(|pauser| deps.api.addr_validate(pauser))
            .collect::<StdResult<Vec<Addr>>>()?,
    )?;
    let res = Response::new()
        .add_attribute("action", "set_pausers")
        .add_attribute("pausers", pausers.join(","));
    Ok(res)
}

pub fn execute_update_royalty_receivers(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    receivers: Vec<WeightedAddress>,
) -> Result<Response, ContractError> {
    // Check if sender is admin
    let collection_details = COLLECTION.load(deps.storage)?;
    let auth_details = AUTH_DETAILS.load(deps.storage)?;
    if info.sender != auth_details.admin {
        return Err(ContractError::Unauthorized {});
    }
    let new_collection_details =
        update_collection_details(&collection_details, None, None, None, Some(receivers));

    COLLECTION.save(deps.storage, &new_collection_details)?;

    let update_denom_msg: CosmosMsg = generate_update_denom_msg(
        &new_collection_details,
        auth_details.payment_collector,
        env.contract.address,
    )?
    .into();

    let res = Response::new()
        .add_message(update_denom_msg)
        .add_attribute("action", "update_royalty_receivers");
    Ok(res)
}

pub fn execute_update_denom(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    collection_name: Option<String>,
    description: Option<String>,
    preview_uri: Option<String>,
) -> Result<Response, ContractError> {
    // Check if sender is admin
    let collection_details = COLLECTION.load(deps.storage)?;
    let auth_details = AUTH_DETAILS.load(deps.storage)?;
    if info.sender != auth_details.admin {
        return Err(ContractError::Unauthorized {});
    }
    let new_collection_details = update_collection_details(
        &collection_details,
        collection_name,
        description,
        preview_uri,
        None,
    );

    COLLECTION.save(deps.storage, &new_collection_details)?;
    // Generate update denom message with the updated collection details
    let update_denom_msg: CosmosMsg = generate_update_denom_msg(
        &new_collection_details,
        auth_details.payment_collector,
        env.contract.address,
    )?
    .into();

    let res = Response::new()
        .add_attribute("action", "update_denom")
        .add_message(update_denom_msg);
    Ok(res)
}
fn execute_purge_denom(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    // Check if sender is admin
    let collection = COLLECTION.load(deps.storage)?;
    let auth_details = AUTH_DETAILS.load(deps.storage)?;
    if info.sender != auth_details.admin {
        return Err(ContractError::Unauthorized {});
    }
    let purge_msg: CosmosMsg = MsgPurgeDenom {
        sender: env.contract.address.into_string(),
        id: collection.id,
    }
    .into();

    let res = Response::new()
        .add_attribute("action", "purge_denom")
        .add_message(purge_msg);
    Ok(res)
}

// Implement Queries
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(
    deps: Deps,
    env: Env,
    msg: BaseMinterQueryMsg<MinterExtensionQueryMsg>,
) -> StdResult<Binary> {
    match msg {
        BaseMinterQueryMsg::Collection {} => to_json_binary(&query_collection(deps, env)?),
        BaseMinterQueryMsg::Config {} => to_json_binary(&query_config(deps, env)?),
        BaseMinterQueryMsg::UserMintingDetails { address } => {
            to_json_binary(&query_user_minting_details(deps, env, address)?)
        }
        BaseMinterQueryMsg::AuthDetails {} => to_json_binary(&query_auth_details(deps, env)?),
        BaseMinterQueryMsg::IsPaused {} => to_json_binary(&query_is_paused(deps, env)?),
        BaseMinterQueryMsg::Pausers {} => to_json_binary(&query_pausers(deps, env)?),
        BaseMinterQueryMsg::TotalMintedCount {} => {
            to_json_binary(&query_total_minted_count(deps, env)?)
        }
        BaseMinterQueryMsg::TokenDetails {} => to_json_binary(&query_token_details(deps, env)?),
        BaseMinterQueryMsg::Extension(ext) => match ext {
            MinterExtensionQueryMsg::MintableTokens {} => {
                to_json_binary(&query_mintable_tokens(deps, env)?)
            }
            MinterExtensionQueryMsg::TotalTokensRemaining {} => {
                to_json_binary(&query_total_tokens(deps, env)?)
            }
        },
    }
}

fn query_collection(deps: Deps, _env: Env) -> Result<CollectionDetails, ContractError> {
    let collection = COLLECTION.load(deps.storage)?;
    Ok(collection)
}
fn query_token_details(deps: Deps, _env: Env) -> Result<TokenDetails, ContractError> {
    let token_details = TOKEN_DETAILS.load(deps.storage)?;
    Ok(token_details)
}

fn query_config(deps: Deps, _env: Env) -> Result<Config, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    Ok(config)
}

fn query_mintable_tokens(deps: Deps, _env: Env) -> Result<Vec<Token>, ContractError> {
    let mut mintable_tokens: Vec<Token> = Vec::new();
    for item in MINTABLE_TOKENS.range(deps.storage, None, None, Order::Ascending) {
        let (_key, value) = item?;

        // Add the (key, value) tuple to the vector
        mintable_tokens.push(value);
    }
    Ok(mintable_tokens)
}

fn query_user_minting_details(
    deps: Deps,
    _env: Env,
    address: String,
) -> Result<UserDetails, ContractError> {
    let address = deps.api.addr_validate(&address)?;
    let user_minting_details = USER_MINTING_DETAILS
        .load(deps.storage, address)
        .unwrap_or_default();
    Ok(user_minting_details)
}

fn query_total_tokens(deps: Deps, _env: Env) -> Result<u32, ContractError> {
    let total_tokens = TOTAL_TOKENS_REMAINING.load(deps.storage)?;
    Ok(total_tokens)
}

fn query_is_paused(deps: Deps, _env: Env) -> Result<bool, ContractError> {
    let pause_state = PauseState::new()?;
    let is_paused = pause_state.is_paused(deps.storage)?;
    Ok(is_paused)
}

fn query_pausers(deps: Deps, _env: Env) -> Result<Vec<Addr>, ContractError> {
    let pause_state = PauseState::new()?;
    let pausers = pause_state.pausers.load(deps.storage).unwrap_or(vec![]);
    Ok(pausers)
}
fn query_total_minted_count(deps: Deps, _env: Env) -> Result<u32, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let total_tokens = config.num_tokens.unwrap_or(0);
    Ok(total_tokens - TOTAL_TOKENS_REMAINING.load(deps.storage)?)
}
fn query_auth_details(deps: Deps, _env: Env) -> Result<AuthDetails, ContractError> {
    let auth_details = AUTH_DETAILS.load(deps.storage)?;
    Ok(auth_details)
}
