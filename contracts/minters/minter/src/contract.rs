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
use minter_types::{
    generate_mint_message, CollectionDetails, QueryMsg as BaseMinterQueryMsg, TokenDetails,
};
use omniflix_minter_factory::msg::QueryMsg::Params as QueryFactoryParams;
use omniflix_minter_factory::msg::{CreateMinterMsg, ParamsResponse};
use omniflix_round_whitelist::msg::ExecuteMsg::PrivateMint;
use whitelist_types::{
    check_if_address_is_member, check_if_whitelist_is_active, check_whitelist_price,
};

use crate::error::ContractError;
use crate::state::{
    COLLECTION, CONFIG, MINTABLE_TOKENS, MINTED_TOKENS, TOKEN_DETAILS, TOTAL_TOKENS_REMAINING,
};
use crate::utils::{
    collect_mintable_tokens, generate_tokens, randomize_token_list, return_random_token,
};
use minter_types::{Config, Token, UserDetails};
use pauser::{PauseState, PAUSED_KEY, PAUSERS_KEY};

use cw2::set_contract_version;
use omniflix_std::types::omniflix::onft::v1beta1::{
    Collection, MsgCreateDenom, MsgPurgeDenom, MsgUpdateDenom, OnftQuerier, WeightedAddress,
};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:omniflix-minter";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(not(test))]
#[allow(dead_code)]
const CREATION_FEE: Uint128 = Uint128::new(0);
#[allow(dead_code)]
#[cfg(not(test))]
const CREATION_FEE_DENOM: &str = "";

#[cfg(test)]
const CREATION_FEE: Uint128 = Uint128::new(100_000_000);
#[cfg(test)]
const CREATION_FEE_DENOM: &str = "uflix";

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: CreateMinterMsg,
) -> Result<Response, ContractError> {
    // Query denom creation fee
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    // Query factory params of instantiator
    // If the instantiator is not a our factory then we wont be able to parse the response
    let _factory_params: ParamsResponse = deps
        .querier
        .query_wasm_smart(info.sender.clone().into_string(), &QueryFactoryParams {})?;

    // This field is implemented only for testing purposes
    let creation_fee_amount = if CREATION_FEE == Uint128::new(0) {
        let onft_querier = OnftQuerier::new(&deps.querier);
        let params = onft_querier.params()?;
        Uint128::from_str(&params.params.unwrap().denom_creation_fee.unwrap().amount)?
    } else {
        CREATION_FEE
    };
    let creation_fee_denom = if CREATION_FEE_DENOM.is_empty() {
        let onft_querier = OnftQuerier::new(&deps.querier);
        let params = onft_querier.params()?;
        params.params.unwrap().denom_creation_fee.unwrap().denom
    } else {
        CREATION_FEE_DENOM.to_string()
    };

    let amount = must_pay(&info, &creation_fee_denom)?;
    // Exact amount must be paid
    if amount != creation_fee_amount {
        return Err(ContractError::InvalidCreationFee {
            expected: amount,
            sent: amount,
        });
    }
    // Check if per address limit is 0
    if msg.init.per_address_limit == 0 {
        return Err(ContractError::PerAddressLimitZero {});
    }
    // Check num_tokens
    if msg.init.num_tokens == 0 {
        return Err(ContractError::InvalidNumTokens {});
    }
    // Check start time
    if msg.init.start_time < env.block.time {
        return Err(ContractError::InvalidStartTime {});
    }
    // Check end time
    if let Some(end_time) = msg.init.end_time {
        if end_time < msg.init.start_time {
            return Err(ContractError::InvalidEndTime {});
        }
    }

    // Check royalty ratio we expect decimal number
    let royalty_ratio = msg.token_details.royalty_ratio;
    if royalty_ratio < Decimal::zero() || royalty_ratio > Decimal::one() {
        return Err(ContractError::InvalidRoyaltyRatio {});
    }
    // Check if whitelist already active
    if let Some(whitelist_address) = msg.init.whitelist_address.clone() {
        let is_active = check_if_whitelist_is_active(
            &deps.api.addr_validate(&whitelist_address)?,
            deps.as_ref(),
        )?;
        if is_active {
            return Err(ContractError::WhitelistAlreadyActive {});
        }
    }

    // Check mint price
    if msg.init.mint_price.amount == Uint128::new(0) {
        return Err(ContractError::InvalidMintPrice {});
    }

    let admin = deps.api.addr_validate(&msg.init.admin)?;

    let payment_collector =
        maybe_addr(deps.api, msg.init.payment_collector.clone())?.unwrap_or(admin.clone());

    let num_tokens = msg.init.num_tokens;

    let config = Config {
        per_address_limit: msg.init.per_address_limit,
        payment_collector: payment_collector.clone(),
        start_time: msg.init.start_time,
        admin: admin.clone(),
        mint_price: msg.init.mint_price,
        whitelist_address: maybe_addr(deps.api, msg.init.whitelist_address.clone())?,
        end_time: msg.init.end_time,
        num_tokens: Some(num_tokens),
    };
    CONFIG.save(deps.storage, &config)?;

    let collection_details = msg.collection_details;
    let token_details = msg.token_details;
    COLLECTION.save(deps.storage, &collection_details.clone())?;
    TOKEN_DETAILS.save(deps.storage, &token_details)?;

    // Generate tokens
    let tokens = generate_tokens(num_tokens);
    let randomized_list = randomize_token_list(tokens.clone(), num_tokens, env.clone())?;

    // Save mintable tokens
    for token in randomized_list {
        MINTABLE_TOKENS.save(deps.storage, token.0, &token.1)?;
    }

    // Save total tokens
    TOTAL_TOKENS_REMAINING.save(deps.storage, &num_tokens)?;

    // Initialize pause state and set admin as pauser
    let pause_state = PauseState::new(PAUSED_KEY, PAUSERS_KEY)?;
    pause_state.set_pausers(deps.storage, info.sender.clone(), [admin.clone()].to_vec())?;

    let collection_creation_msg: CosmosMsg = MsgCreateDenom {
        // Option
        description: collection_details.description.unwrap_or("".to_string()),
        id: collection_details.id,
        name: collection_details.collection_name,
        preview_uri: collection_details.preview_uri.unwrap_or("".to_string()),
        schema: collection_details.schema.unwrap_or("".to_string()),
        sender: env.contract.address.into_string(),
        symbol: collection_details.symbol,
        data: collection_details.data.unwrap_or("".to_string()),
        uri: collection_details.uri.unwrap_or("".to_string()),
        uri_hash: collection_details.uri_hash.unwrap_or("".to_string()),
        creation_fee: Some(
            Coin {
                denom: creation_fee_denom,
                amount: creation_fee_amount,
            }
            .into(),
        ),
        royalty_receivers: collection_details
            .royalty_receivers
            .unwrap_or(vec![WeightedAddress {
                address: payment_collector.clone().into_string(),
                weight: Decimal::one().to_string(),
            }]),
    }
    .into();

    let res = Response::new()
        .add_message(collection_creation_msg)
        .add_attribute("action", "instantiate");

    Ok(res)
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
        ExecuteMsg::Pause {} => execute_pause(deps, env, info),
        ExecuteMsg::Unpause {} => execute_unpause(deps, env, info),
        ExecuteMsg::SetPausers { pausers } => execute_set_pausers(deps, env, info, pausers),
        ExecuteMsg::UpdateRoyaltyReceivers { receivers } => {
            execute_update_royalty_receivers(deps, env, info, receivers)
        }
        ExecuteMsg::UpdateDenom {
            name,
            description,
            preview_uri,
        } => execute_update_denom(deps, env, info, name, description, preview_uri),
        ExecuteMsg::PurgeDenom {} => execute_purge_denom(deps, env, info),
    }
}

pub fn execute_mint(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    // Error if paused
    let pause_state = PauseState::new(PAUSED_KEY, PAUSERS_KEY)?;
    pause_state.error_if_paused(deps.storage)?;
    // Load config
    let config = CONFIG.load(deps.storage)?;
    // Check if any tokens are left
    let total_tokens_remaining = TOTAL_TOKENS_REMAINING.load(deps.storage)?;

    if total_tokens_remaining == 0 {
        return Err(ContractError::NoTokensLeftToMint {});
    }
    let mut user_details = MINTED_TOKENS
        .may_load(deps.storage, info.sender.clone())?
        .unwrap_or(UserDetails::default());

    let mut mint_price = config.mint_price;

    // Collect mintable tokens
    let mintable_tokens = collect_mintable_tokens(deps.as_ref().storage)?;

    // Check if public end time is determined and if it is passed
    if let Some(end_time) = config.end_time {
        if env.block.time > end_time {
            return Err(ContractError::PublicMintingEnded {});
        }
    }

    // Get a random token id
    let random_token = return_random_token(&mintable_tokens, env.clone())?;

    // Check if minting is started
    let is_public = env.block.time > config.start_time;

    let mut messages: Vec<CosmosMsg> = vec![];

    if !is_public {
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
        user_details.public_mint_count += 1;
        // Check if address has reached the limit
        // This check should be done only for public minting
        if user_details.public_mint_count > config.per_address_limit {
            return Err(ContractError::AddressReachedMintLimit {});
        }
    }
    // Increment total minted count
    user_details.total_minted_count += 1;
    // Add the minted token to the user details
    user_details.minted_tokens.push(random_token.1.clone());

    MINTED_TOKENS.save(deps.storage, info.sender.clone(), &user_details)?;

    // Check the payment
    let amount = may_pay(&info, &mint_price.denom)?;
    // Exact amount must be paid
    if amount != mint_price.amount {
        return Err(ContractError::IncorrectPaymentAmount {
            expected: mint_price.amount,
            sent: amount,
        });
    }
    // Get the payment collector address
    let payment_collector = config.payment_collector;
    let collection = COLLECTION.load(deps.storage)?;
    let token_details = TOKEN_DETAILS.load(deps.storage)?;
    // Update storage
    // Remove the token from the mintable tokens
    MINTABLE_TOKENS.remove(deps.storage, random_token.0);
    // Decrement the total tokens remaining
    TOTAL_TOKENS_REMAINING.update(deps.storage, |mut total_tokens| -> StdResult<_> {
        total_tokens -= 1;
        Ok(total_tokens)
    })?;
    // Save the user details
    let token_id = random_token.1.token_id;

    let mint_msg: CosmosMsg = generate_mint_message(
        &collection,
        &token_details,
        token_id.clone(),
        env.contract.address,
        info.sender,
        None,
        false,
    )
    .into();

    if !mint_price.amount.is_zero() {
        // Create the Bank send message
        let bank_msg: CosmosMsg = CosmosMsg::Bank(cosmwasm_std::BankMsg::Send {
            to_address: payment_collector.into_string(),
            amount: vec![Coin {
                denom: mint_price.denom,
                amount: mint_price.amount,
            }],
        });

        messages.push(bank_msg.clone());
    }

    messages.push(mint_msg.clone());

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
    let pause_state = PauseState::new(PAUSED_KEY, PAUSERS_KEY)?;
    pause_state.error_if_paused(deps.storage)?;
    // Check if sender is admin
    nonpayable(&info)?;
    let config = CONFIG.load(deps.storage)?;
    let collection = COLLECTION.load(deps.storage)?;
    if info.sender != config.admin {
        return Err(ContractError::Unauthorized {});
    }
    let recipient = deps.api.addr_validate(&recipient)?;

    // Collect mintable tokens
    let mintable_tokens = collect_mintable_tokens(deps.as_ref().storage)?;

    let token = match token_id {
        None => return_random_token(&mintable_tokens, env.clone())?,
        Some(token_id) => {
            // Find key for the desired token
            let token: Option<(u32, Token)> = mintable_tokens
                .iter()
                .find(|(_, token)| token.token_id == token_id)
                .map(|(key, token)| (*key, token.clone()));

            match token {
                None => {
                    return Err(ContractError::TokenIdNotMintable {});
                }
                Some(token) => token,
            }
        }
    };
    // Remove the token from the mintable tokens
    MINTABLE_TOKENS.remove(deps.storage, token.0);

    // Decrement the total tokens
    TOTAL_TOKENS_REMAINING.update(deps.storage, |mut total_tokens| -> StdResult<_> {
        total_tokens -= 1;
        Ok(total_tokens)
    })?;

    // Increment the minted tokens for the addres
    let mut user_details = MINTED_TOKENS
        .may_load(deps.storage, recipient.clone())?
        .unwrap_or(UserDetails::default());
    // We are updating parameter ourself and not using add_minted_token function because we want to override per address limit checks
    user_details.minted_tokens.push(token.1.clone());
    user_details.total_minted_count += 1;
    // Save details
    MINTED_TOKENS.save(deps.storage, recipient.clone(), &user_details)?;

    let token_id = token.1.token_id;

    let mint_msg: CosmosMsg = generate_mint_message(
        &collection,
        &TOKEN_DETAILS.load(deps.storage)?,
        token_id.clone(),
        env.contract.address,
        recipient.clone(),
        None,
        false,
    )
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
    let config = CONFIG.load(deps.storage)?;

    if info.sender != config.admin {
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
    env: Env,
    info: MessageInfo,
    ratio: String,
) -> Result<Response, ContractError> {
    // Check if sender is admin
    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.admin {
        return Err(ContractError::Unauthorized {});
    }
    // Check if trading has started
    if env.block.time > config.start_time {
        return Err(ContractError::MintingAlreadyStarted {});
    }
    let ratio = Decimal::from_str(&ratio)?; // Check if ratio is decimal number
    if ratio < Decimal::zero() || ratio > Decimal::one() {
        return Err(ContractError::InvalidRoyaltyRatio {});
    }
    let mut token_details = TOKEN_DETAILS.load(deps.storage)?;
    token_details.royalty_ratio = ratio;
    TOKEN_DETAILS.save(deps.storage, &token_details)?;

    CONFIG.save(deps.storage, &config)?;

    let res = Response::new()
        .add_attribute("action", "update_royalty_ratio")
        .add_attribute("ratio", ratio.to_string());
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
    if info.sender != config.admin {
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
    let config = CONFIG.load(deps.storage)?;
    // This should be available for everyone but then this could be abused
    if info.sender != config.admin {
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
    if info.sender != config.admin {
        return Err(ContractError::Unauthorized {});
    }
    let whitelist_address = config.whitelist_address.clone();
    // In order to update whitelist address we first check if currently there is any whitelist address
    // If there is whitelist address then we check if it is active
    // If it is active then we throw error because creator can not update active whitelist address
    // If it is not active then we check if the address that creator wants to update is active
    // If it is active then we throw error because creator can not set a whitelist address that is already active
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
    let pause_state = PauseState::new(PAUSED_KEY, PAUSERS_KEY)?;
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
    let pause_state = PauseState::new(PAUSED_KEY, PAUSERS_KEY)?;
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
    let pause_state = PauseState::new(PAUSED_KEY, PAUSERS_KEY)?;
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
    let mut collection = COLLECTION.load(deps.storage)?;
    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.admin {
        return Err(ContractError::Unauthorized {});
    }
    collection.royalty_receivers = Some(receivers.clone());

    COLLECTION.save(deps.storage, &collection)?;

    let update_msg: CosmosMsg = MsgUpdateDenom {
        sender: env.contract.address.into_string(),
        royalty_receivers: receivers,
        id: collection.id,
        description: "[do-not-modify]".to_string(),
        name: "[do-not-modify]".to_string(),
        preview_uri: "[do-not-modify]".to_string(),
    }
    .into();

    let res = Response::new()
        .add_message(update_msg)
        .add_attribute("action", "update_royalty_receivers");
    Ok(res)
}

pub fn execute_update_denom(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    name: Option<String>,
    description: Option<String>,
    preview_uri: Option<String>,
) -> Result<Response, ContractError> {
    // Check if sender is admin
    let mut collection = COLLECTION.load(deps.storage)?;
    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.admin {
        return Err(ContractError::Unauthorized {});
    }

    collection.collection_name = name.clone().unwrap_or(collection.collection_name);
    collection.description = description.clone();
    collection.preview_uri = preview_uri.clone();
    COLLECTION.save(deps.storage, &collection)?;

    let update_msg: CosmosMsg = MsgUpdateDenom {
        sender: env.contract.address.into_string(),
        id: collection.id,
        description: description.unwrap_or("[do-not-modify]".to_string()),
        name: name.unwrap_or("[do-not-modify]".to_string()),
        preview_uri: preview_uri.unwrap_or("[do-not-modify]".to_string()),
        royalty_receivers: [].to_vec(),
    }
    .into();

    let res = Response::new()
        .add_attribute("action", "update_denom")
        .add_message(update_msg);
    Ok(res)
}
fn execute_purge_denom(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    // Check if sender is admin
    let collection = COLLECTION.load(deps.storage)?;
    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.admin {
        return Err(ContractError::Unauthorized {});
    }
    let onft_querier = OnftQuerier::new(&deps.querier);
    let minted_nfties_res = onft_querier.collection(collection.clone().id, None)?;
    let minted_nfties = minted_nfties_res
        .collection
        .unwrap_or(Collection::default())
        .onfts;
    if !minted_nfties.is_empty() {
        // If there is any nft minted for the collection update denoms should not work
        return Err(ContractError::MintingAlreadyStarted {});
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
        BaseMinterQueryMsg::MintedTokens { address } => {
            to_json_binary(&query_minted_tokens(deps, env, address)?)
        }
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

fn query_minted_tokens(
    deps: Deps,
    _env: Env,
    address: String,
) -> Result<UserDetails, ContractError> {
    let address = deps.api.addr_validate(&address)?;
    let minted_tokens = MINTED_TOKENS.load(deps.storage, address)?;
    Ok(minted_tokens)
}

fn query_total_tokens(deps: Deps, _env: Env) -> Result<u32, ContractError> {
    let total_tokens = TOTAL_TOKENS_REMAINING.load(deps.storage)?;
    Ok(total_tokens)
}

fn query_is_paused(deps: Deps, _env: Env) -> Result<bool, ContractError> {
    let pause_state = PauseState::new(PAUSED_KEY, PAUSERS_KEY)?;
    let is_paused = pause_state.is_paused(deps.storage)?;
    Ok(is_paused)
}

fn query_pausers(deps: Deps, _env: Env) -> Result<Vec<Addr>, ContractError> {
    let pause_state = PauseState::new(PAUSED_KEY, PAUSERS_KEY)?;
    let pausers = pause_state.pausers.load(deps.storage).unwrap_or(vec![]);
    Ok(pausers)
}
fn query_total_minted_count(deps: Deps, _env: Env) -> Result<u32, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let total_tokens = config.num_tokens.unwrap_or(0);
    Ok(total_tokens - TOTAL_TOKENS_REMAINING.load(deps.storage)?)
}
