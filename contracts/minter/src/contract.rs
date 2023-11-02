use std::str::FromStr;

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Addr, Binary, Coin, CosmosMsg, Decimal, Deps, DepsMut, Env, MessageInfo, Order,
    Response, StdResult, Uint128,
};

use types::whitelist::{
    HasMemberResponse, IsActiveResponse, PerAddressLimitResponse, WhitelistQueryMsgs,
};
// Use this as whitelist config
use types::whitelist::Config as WhitelistConfig;

use cw_utils::{maybe_addr, must_pay, nonpayable};

use crate::msg::{CollectionDetails, ExecuteMsg, InstantiateMsg, QueryMsg, WhitelistQueryMsg};

use crate::error::ContractError;
use crate::state::{
    Config, Round, Token, UserDetails, COLLECTION, CONFIG, MINTABLE_TOKENS, MINTED_TOKENS, ROUNDS,
    TOTAL_TOKENS_REMAINING,
};
use crate::utils::{
    check_if_whitelisted, check_round_overlaps, find_active_round, randomize_token_list,
    return_random_token_id, return_updated_round,
};

use cw2::set_contract_version;
use omniflix_std::types::omniflix::onft::v1beta1::{
    Metadata, MsgCreateDenom, MsgMintOnft, OnftQuerier,
};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:omniflix-minter";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(not(test))]
const CREATION_FEE: Uint128 = Uint128::new(0);
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
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    // Query denom creation fee
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    // This field is implemented only for testing purposes
    let creation_fee_amount = if CREATION_FEE == Uint128::new(0) {
        let onft_querier = OnftQuerier::new(&deps.querier);
        let params = onft_querier.params()?;
        Uint128::from_str(&params.params.unwrap().denom_creation_fee.unwrap().amount)?
    } else {
        CREATION_FEE
    };
    let creation_fee_denom = if CREATION_FEE_DENOM == "" {
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
    // Check per address limit is not 0
    if msg.per_address_limit == 0 {
        return Err(ContractError::PerAddressLimitZero {});
    }

    // Check num_tokens
    if msg.collection_details.num_tokens == 0 {
        return Err(ContractError::InvalidNumTokens {});
    }
    // Check start time
    if msg.start_time < env.block.time {
        return Err(ContractError::InvalidStartTime {});
    }

    // Check royalty ratio we expect decimal number
    let royalty_ratio = Decimal::from_str(&msg.royalty_ratio)?;
    if royalty_ratio < Decimal::zero() || royalty_ratio > Decimal::one() {
        return Err(ContractError::InvalidRoyaltyRatio {});
    }

    if royalty_ratio > Decimal::one() {
        return Err(ContractError::InvalidRoyaltyRatio {});
    }
    if msg.rounds.is_some() {
        // First update the rounds. We are only updating whitelist rounds
        let mut rounds = msg.rounds.unwrap();
        let mut updated_rounds: Vec<(u32, Round)>;
        for mut round in rounds {
            let mut i = 1;
            let updated = return_updated_round(deps, round)?;
            updated_rounds.push((i, round));
        }

        // Check if the rounds overlap if none we can save it
        check_round_overlaps(env.block.time, updated_rounds, msg.start_time)?;
        // Save the rounds
        for round in updated_rounds {
            ROUNDS.save(deps.storage, round.0, &round.1)?;
        }
    }
    // Check mint price
    if msg.mint_price == Uint128::new(0) {
        return Err(ContractError::InvalidMintPrice {});
    }

    let creator = maybe_addr(deps.api, msg.creator.clone())?.unwrap_or(info.sender.clone());
    let payment_collector =
        maybe_addr(deps.api, msg.payment_collector.clone())?.unwrap_or(info.sender.clone());
    let num_tokens = msg.collection_details.num_tokens;
    let config = Config {
        per_address_limit: msg.per_address_limit,
        payment_collector: payment_collector,
        mint_denom: msg.mint_denom,
        start_time: msg.start_time,
        mint_price: msg.mint_price,
        royalty_ratio: royalty_ratio,
        creator: creator,
    };
    CONFIG.save(deps.storage, &config)?;

    let collection = CollectionDetails {
        name: msg.collection_details.name,
        description: msg.collection_details.description,
        preview_uri: msg.collection_details.preview_uri,
        schema: msg.collection_details.schema,
        symbol: msg.collection_details.symbol,
        id: msg.collection_details.id,
        num_tokens: msg.collection_details.num_tokens,
        extensible: msg.collection_details.extensible,
        nsfw: msg.collection_details.nsfw,
        base_uri: msg.collection_details.base_uri,
    };
    COLLECTION.save(deps.storage, &collection)?;

    // Generate tokens
    let tokens: Vec<(u32, Token)> = (1..=num_tokens)
        .map(|x| {
            (
                x,
                Token {
                    token_id: x.to_string(),
                },
            )
        })
        .collect();

    let randomized_list = randomize_token_list(tokens, num_tokens, env.clone())?;
    // Save mintable tokens
    randomized_list.into_iter().for_each(|(key, value)| {
        MINTABLE_TOKENS
            .save(deps.storage, key, &value)
            // TODO Fix here
            .unwrap_or_else(|_| {
                panic!(
                    "Unable to save mintable tokens with key {} and value {}",
                    key, value.token_id
                )
            });
    });
    // Save total tokens
    TOTAL_TOKENS_REMAINING.save(deps.storage, &num_tokens)?;

    let nft_creation_msg: CosmosMsg = MsgCreateDenom {
        description: collection.description,
        id: collection.id,
        name: collection.name,
        preview_uri: collection.preview_uri,
        schema: collection.schema,
        sender: env.contract.address.into_string(),
        symbol: collection.symbol,
        creation_fee: Some(
            Coin {
                denom: creation_fee_denom,
                amount: creation_fee_amount,
            }
            .into(),
        ),
    }
    .into();

    let res = Response::new()
        .add_message(nft_creation_msg)
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
        ExecuteMsg::Mint {} => execute_mint(deps, env, info, msg),
        ExecuteMsg::MintAdmin {
            recipient,
            denom_id,
        } => execute_mint_admin(deps, env, info, recipient, denom_id),
        ExecuteMsg::BurnRemainingTokens {} => execute_burn_remaining_tokens(deps, env, info),
        ExecuteMsg::UpdateRoyaltyRatio { ratio } => {
            execute_update_royalty_ratio(deps, env, info, ratio)
        }
        ExecuteMsg::UpdateMintPrice { mint_price } => {
            execute_update_mint_price(deps, env, info, mint_price)
        }
        ExecuteMsg::RandomizeList {} => execute_randomize_list(deps, env, info),
        ExecuteMsg::RemoveRound { round_index } => {
            execute_remove_round(deps, env, info, round_index)
        }
        ExecuteMsg::AddRound { round } => execute_add_round(deps, env, info, round),
        ExecuteMsg::UpdateRound { round_index } => {
            execute_update_round(deps, env, info, round_index)
        }
    }
}

pub fn execute_mint(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    _msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    // Check if any tokens are left
    let total_tokens_remaining = TOTAL_TOKENS_REMAINING.load(deps.storage)?;

    if total_tokens_remaining == 0 {
        return Err(ContractError::NoTokensLeftToMint {});
    }
    let config = CONFIG.load(deps.storage)?;
    let mut mint_price = config.mint_price;
    // Collect mintable tokens
    let mut mintable_tokens: Vec<(u32, Token)> = Vec::new();
    for item in MINTABLE_TOKENS.range(deps.storage, None, None, Order::Ascending) {
        let (key, value) = item?;

        // Add the (key, value) tuple to the vector
        mintable_tokens.push((key, value));
    }

    // Get a random token id
    let random_token = return_random_token_id(&mintable_tokens, env.clone())?;
    // Check if minting is started
    if env.block.time < config.start_time {
        // If not public mint try to find the rounds
        let rounds: StdResult<Vec<(u32, Round)>> = ROUNDS
            .range(deps.storage, None, None, Order::Ascending)
            .collect();
        let rounds = rounds.unwrap_or(Vec::new());
        if rounds.is_empty() {
            return Err(ContractError::MintingNotStarted {
                start_time: config.start_time.seconds(),
                current_time: env.block.time.seconds(),
            });
        }
        // First check if rounds overlap
        check_round_overlaps(env.block.time, rounds, config.start_time)?;
        // Check if any active round exists
        let active_round = find_active_round(env.block.time, rounds)?;
        let active_round_index = active_round.0;
        // Check if the address is whitelisted
        let is_member = check_if_whitelisted(
            info.sender.clone().into_string(),
            active_round.1,
            deps.as_ref(),
        )?;
        let mut user_details = MINTED_TOKENS
            .may_load(deps.storage, info.sender)?
            .unwrap_or(UserDetails::new());
        // This function tries to add mintable token to user details
        // If succesfully updates it that means per_address_limit or round limit is not reached
        user_details.add_minted_token(
            config.per_address_limit,
            Some(active_round.1.per_address_limit()),
            random_token.1,
            Some(active_round_index),
        )?;
        MINTED_TOKENS.save(deps.storage, info.sender, &user_details)?;
        // Determine mint price
        mint_price = active_round.1.mint_price()
    } else {
        // Check if the address has reached the limit
        let mut user_details = MINTED_TOKENS
            .may_load(deps.storage, info.sender)?
            .unwrap_or(UserDetails::new());
        user_details.add_minted_token(config.per_address_limit, None, random_token.1, None)?;
        // Save new data
        MINTED_TOKENS.save(deps.storage, info.sender, &user_details)?;
    }

    // Check the payment
    let amount = must_pay(&info, &config.mint_denom)?;
    if amount != mint_price {
        return Err(ContractError::IncorrectPaymentAmount {
            expected: mint_price,
            sent: amount,
        });
    }
    // Get the payment collector address
    let payment_collector = config.payment_collector;
    let collection = COLLECTION.load(deps.storage)?;
    // Update storage
    // Remove the token from the mintable tokens
    MINTABLE_TOKENS.remove(deps.storage, random_token.0);
    // Decrement the total tokens remaining
    TOTAL_TOKENS_REMAINING.update(deps.storage, |mut total_tokens| -> StdResult<_> {
        total_tokens -= 1;
        Ok(total_tokens)
    })?;
    let token_id = random_token.1.token_id;
    // Generate the metadata
    let metadata = Metadata {
        name: format!("{} # {}", collection.name, token_id),
        description: collection.description,
        media_uri: format!("{}/{}", collection.base_uri, token_id),
        preview_uri: collection.preview_uri,
    };
    // Create the mint message
    let mint_msg: CosmosMsg = MsgMintOnft {
        data: "".to_string(),
        id: format!("{}{}", collection.id, token_id),
        metadata: Some(metadata),
        denom_id: collection.id.clone(),
        transferable: true,
        sender: env.contract.address.clone().into_string(),
        extensible: collection.extensible,
        nsfw: collection.nsfw,
        recipient: info.sender.clone().into_string(),
        royalty_share: config.royalty_ratio.atomics().to_string(),
    }
    .into();

    // Create the Bank send message
    let bank_msg: CosmosMsg = CosmosMsg::Bank(cosmwasm_std::BankMsg::Send {
        to_address: payment_collector.into_string(),
        amount: vec![Coin {
            denom: config.mint_denom,
            amount: config.mint_price,
        }],
    })
    .into();

    let res = Response::new()
        .add_message(mint_msg)
        .add_message(bank_msg)
        .add_attribute("action", "mint")
        .add_attribute("token_id", token_id.to_string())
        .add_attribute("denom_id", token_id.to_string());

    Ok(res)
}

pub fn execute_mint_admin(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    recipient: String,
    denom_id: Option<String>,
) -> Result<Response, ContractError> {
    // Check if sender is admin
    nonpayable(&info)?;
    let config = CONFIG.load(deps.storage)?;
    let collection = COLLECTION.load(deps.storage)?;
    if info.sender != config.creator {
        return Err(ContractError::Unauthorized {});
    }
    let recipient = deps.api.addr_validate(&recipient)?;

    // Collect mintable tokens
    let mut mintable_tokens: Vec<(u32, Token)> = Vec::new();
    for item in MINTABLE_TOKENS.range(deps.storage, None, None, Order::Ascending) {
        let (key, value) = item?;
        // Add the (key, value) tuple to the vector
        mintable_tokens.push((key, value));
    }
    let token = match denom_id {
        None => return_random_token_id(&mintable_tokens, env.clone())?,
        Some(denom_id) => {
            // Find key for the desired token
            let token: Option<(u32, Token)> = mintable_tokens
                .iter()
                .find(|(_, token)| token.token_id == denom_id)
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
        .unwrap_or(UserDetails::new());
    // We are updating parameter ourself and not using add_minted_token function because we want to override per address limit checks
    user_details.minted_tokens.push(token.1);
    user_details.total_minted_count += 1;
    // Save details
    MINTED_TOKENS.save(deps.storage, recipient, &user_details);

    let denom_id = token.1.token_id;

    // Generate the metadata
    let metadata = Metadata {
        name: format!("{} # {}", collection.name, denom_id),
        description: collection.description,
        media_uri: format!("{}/{}", collection.preview_uri, denom_id),
        preview_uri: collection.preview_uri,
    };

    // Create the mint message
    let mint_msg: CosmosMsg = MsgMintOnft {
        data: "".to_string(),
        id: format!("{}{}", collection.id, denom_id),
        metadata: Some(metadata),
        denom_id: collection.id.clone(),
        transferable: true,
        sender: env.contract.address.into_string(),
        extensible: collection.extensible,
        nsfw: collection.nsfw,
        recipient: recipient.into_string(),
        royalty_share: config.royalty_ratio.atomics().to_string(),
    }
    .into();

    let res = Response::new()
        .add_message(mint_msg)
        .add_attribute("action", "mint")
        .add_attribute("token_id", denom_id.to_string())
        .add_attribute("denom_id", denom_id.to_string());
    Ok(res)
}

pub fn execute_set_whitelist(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    address: String,
) -> Result<Response, ContractError> {
    // Check if sender is admin
    let mut config = CONFIG.load(deps.storage)?;
    if info.sender != config.creator {
        return Err(ContractError::Unauthorized {});
    }

    let address = deps.api.addr_validate(&address)?;

    CONFIG.save(deps.storage, &new_config)?;

    let res = Response::new()
        .add_attribute("action", "set_whitelist")
        .add_attribute("address", address.to_string());
    Ok(res)
}

pub fn execute_burn_remaining_tokens(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    // Check if sender is admin
    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.creator {
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
    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.creator {
        return Err(ContractError::Unauthorized {});
    }
    let ratio = Decimal::from_str(&ratio)?; // Check if ratio is decimal number
    if ratio < Decimal::zero() || ratio > Decimal::one() {
        return Err(ContractError::InvalidRoyaltyRatio {});
    }

    let new_config = Config {
        per_address_limit: config.per_address_limit,
        payment_collector: config.payment_collector,
        whitelist_address: config.whitelist_address,
        mint_denom: config.mint_denom,
        start_time: config.start_time,
        mint_price: config.mint_price,
        royalty_ratio: ratio,
        creator: config.creator,
    };

    CONFIG.save(deps.storage, &new_config)?;

    let res = Response::new()
        .add_attribute("action", "update_royalty_ratio")
        .add_attribute("ratio", ratio.to_string());
    Ok(res)
}

pub fn execute_update_mint_price(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    mint_price: Uint128,
) -> Result<Response, ContractError> {
    // Check if sender is admin
    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.creator {
        return Err(ContractError::Unauthorized {});
    }
    // Check if trading has started
    if env.block.time > config.start_time {
        return Err(ContractError::MintingAlreadyStarted {});
    }

    let new_config = Config {
        per_address_limit: config.per_address_limit,
        payment_collector: config.payment_collector,
        whitelist_address: config.whitelist_address,
        mint_denom: config.mint_denom,
        start_time: config.start_time,
        mint_price,
        royalty_ratio: config.royalty_ratio,
        creator: config.creator,
    };

    CONFIG.save(deps.storage, &new_config)?;

    let res = Response::new()
        .add_attribute("action", "update_mint_price")
        .add_attribute("mint_price", mint_price.to_string());
    Ok(res)
}

pub fn execute_randomize_list(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    // Check if sender is admin
    let collection = COLLECTION.load(deps.storage)?;
    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.creator {
        return Err(ContractError::Unauthorized {});
    }

    // Collect mintable tokens
    let mut mintable_tokens: Vec<(u32, Token)> = Vec::new();
    for item in MINTABLE_TOKENS.range(deps.storage, None, None, Order::Ascending) {
        let (key, value) = item?;

        // Add the (key, value) tuple to the vector
        mintable_tokens.push((key, value));
    }

    let randomized_list = randomize_token_list(mintable_tokens, collection.num_tokens, env)?;

    for token in randomized_list {
        MINTABLE_TOKENS.save(deps.storage, token.0, &token.1)?;
    }

    let res = Response::new().add_attribute("action", "randomize_list");
    Ok(res)
}

pub fn execute_remove_round(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    round_index: u32,
) -> Result<Response, ContractError> {
    // Check if sender is admin
    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.creator {
        return Err(ContractError::Unauthorized {});
    }
    // Check if the round exists
    let round = ROUNDS.may_load(deps.storage, round_index)?;
    if round.is_none() {
        return Err(ContractError::RoundNotFound {});
    }
    // Remove the round
    ROUNDS.remove(deps.storage, round_index);

    let res = Response::new()
        .add_attribute("action", "remove_round")
        .add_attribute("round_index", round_index.to_string());
    Ok(res)
}

pub fn execute_add_round(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    round: Round,
) -> Result<Response, ContractError> {
    // Check if sender is admin
    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.creator {
        return Err(ContractError::Unauthorized {});
    }
    // Check if the round exists
    let mut rounds = ROUNDS
        .range(deps.storage, None, None, Order::Ascending)
        .collect::<StdResult<Vec<(u32, Round)>>>()?;
    let round_exists = rounds.iter().any(|(_, r)| r == &round);
    if round_exists {
        return Err(ContractError::RoundAlreadyExists {});
    }
    let round_index = rounds.len() as u32 + 1;
    rounds.push((round_index, round.clone()));
    // Check if the round start time is valid
    if round.start_time() < env.block.time {
        return Err(ContractError::RoundStartTimeInvalid {});
    }
    // Check if rounds overlap
    check_round_overlaps(env.block.time, rounds, config.start_time)?;

    // Save the round
    ROUNDS.save(deps.storage, round_index, &round)?;

    let res = Response::new()
        .add_attribute("action", "add_round")
        .add_attribute("round_index", round_index.to_string());

    Ok(res)
}

// Implement Queries
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Collection {} => to_json_binary(&query_collection(deps, env)?),
        QueryMsg::Config {} => to_json_binary(&query_config(deps, env)?),
        QueryMsg::MintableTokens {} => to_json_binary(&query_mintable_tokens(deps, env)?),
        QueryMsg::MintedTokens { address } => {
            to_json_binary(&query_minted_tokens(deps, env, address)?)
        }
        QueryMsg::TotalTokens {} => to_json_binary(&query_total_tokens(deps, env)?),
        QueryMsg::Whitelist {} => to_json_binary(&query_whitelist(deps, env)?),
    }
}

fn query_collection(deps: Deps, _env: Env) -> Result<CollectionDetails, ContractError> {
    let collection = COLLECTION.load(deps.storage)?;
    Ok(collection)
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
) -> Result<Vec<Token>, ContractError> {
    let address = deps.api.addr_validate(&address)?;
    let minted_tokens = MINTED_TOKENS.load(deps.storage, address)?;
    Ok(minted_tokens)
}

fn query_total_tokens(deps: Deps, _env: Env) -> Result<u32, ContractError> {
    let total_tokens = TOTAL_TOKENS_REMAINING.load(deps.storage)?;
    Ok(total_tokens)
}

fn query_whitelist(deps: Deps, _env: Env) -> Result<Option<String>, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let whitelist_address = match config.whitelist_address {
        Some(address) => Some(address.to_string()),
        None => None,
    };
    // Implement whitelist contract query
    Ok(whitelist_address)
}
