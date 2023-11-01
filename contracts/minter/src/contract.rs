use std::fmt::format;
use std::str::FromStr;

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    from_binary, to_json_binary, Addr, Binary, Coin, CosmosMsg, Decimal, Deps, DepsMut, Env,
    MessageInfo, Order, Response, StdResult, Uint128,
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
    Config, Round, Token, COLLECTION, CONFIG, MINTABLE_TOKENS, MINTED_TOKENS, ROUNDS,
    TOTAL_TOKENS_REMAINING,
};
use crate::utils::{
    check_mint_limit_for_addr, check_whitelist, randomize_token_list, return_random_token_id,
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
    // Check rounds
    if msg.rounds.is_some() {
        let mut whitelist_collections: Vec<Round>;
        let mut whitelist_addresses: Vec<Addr>;
        let rounds = msg.rounds.unwrap();
        rounds.into_iter().map(|x| match x {
            Round::WhitelistAddress {
                address,
                start_time,
                end_time,
            } => {
                whitelist_addresses.push(deps.api.addr_validate(&address.as_str()).unwrap());
            }
            Round::WhitelistCollection {
                collection_id,
                start_time,
                end_time,
                mint_price,
                per_address_limit,
            } => {
                whitelist_collections.push(Round::WhitelistCollection {
                    collection_id,
                    start_time,
                    end_time,
                    mint_price,
                    per_address_limit,
                });
            }
        });
        // Check if these addresses whitelist contracts
        for address in whitelist_addresses {
            // Check if address is a whitelist contract by parsing the config response
            let whitelist_config: WhitelistConfig = deps
                .querier
                .query_wasm_smart(address.clone(), &WhitelistQueryMsg::Config {})?;
            let is_active = env.block.time < whitelist_config.end_time
                && env.block.time > whitelist_config.start_time;
            if is_active {
                return Err(ContractError::WhitelistAlreadyActive {});
            }
        }
        // Check collection parameters are valid
        for collection in whitelist_collections {
            if let Round::WhitelistCollection {
                collection_id,
                start_time,
                end_time,
                mint_price,
                per_address_limit,
            } = collection
            {
                let is_active = env.block.time < end_time && env.block.time > start_time;
                if is_active {
                    return Err(ContractError::WhitelistAlreadyActive {});
                }
                if per_address_limit == 0 {
                    return Err(ContractError::PerAddressLimitZero {});
                }
            }
        }
        // Save rounds using index
        for round in msg.rounds.unwrap() {
            let mut index = 1;
            ROUNDS.save(deps.storage, index, &round)?;
            index += 1;
        }
    }

    // Check royalty ratio we expect decimal number
    let royalty_ratio = Decimal::from_str(&msg.royalty_ratio)?;
    if royalty_ratio < Decimal::zero() || royalty_ratio > Decimal::one() {
        return Err(ContractError::InvalidRoyaltyRatio {});
    }

    if royalty_ratio > Decimal::one() {
        return Err(ContractError::InvalidRoyaltyRatio {});
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
        ExecuteMsg::SetWhitelist { address } => execute_set_whitelist(deps, env, info, address),
        ExecuteMsg::BurnRemainingTokens {} => execute_burn_remaining_tokens(deps, env, info),
        ExecuteMsg::UpdateRoyaltyRatio { ratio } => {
            execute_update_royalty_ratio(deps, env, info, ratio)
        }
        ExecuteMsg::UpdateMintPrice { mint_price } => {
            execute_update_mint_price(deps, env, info, mint_price)
        }
        ExecuteMsg::RandomizeList {} => execute_randomize_list(deps, env, info),
    }
}

pub fn execute_mint(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    _msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    // Check if the minting has started
    let config = CONFIG.load(deps.storage)?;
    // Load rounds if they exist
    let rounds: StdResult<Vec<(u32, Round)>> = ROUNDS
        .range(deps.storage, None, None, Order::Ascending)
        .collect();

    // Check if the rounds were successfully loaded
    let rounds = match rounds {
        Ok(rounds) => Some(rounds),
        Err(_) => None,
    };
    let mint_price: Uint128 = config.mint_price;

    if env.block.time < config.start_time {
        let active_round: (u32, Round) = match rounds {
            Some(rounds) => {
                // First find all active rounds
                let active_rounds: Vec<(u32, Round)>;
                for round_with_index in rounds {
                    match round_with_index.1 {
                        Round::WhitelistAddress {
                            address,
                            start_time,
                            end_time,
                        } => {
                            // Query the Config of the whitelist contract
                            let whitelist_config: WhitelistConfig = deps.querier.query_wasm_smart(
                                address.clone(),
                                &WhitelistQueryMsgs::Config {},
                            )?;
                            let is_active = env.block.time < whitelist_config.end_time
                                && env.block.time > whitelist_config.start_time;
                            if is_active {
                                let round = Round::WhitelistAddress {
                                    address,
                                    start_time: Some(whitelist_config.start_time),
                                    end_time: Some(whitelist_config.end_time),
                                };
                                active_rounds.push((round_with_index.0, round));
                            }
                        }
                        Round::WhitelistCollection {
                            collection_id,
                            start_time,
                            end_time,
                            mint_price,
                            per_address_limit,
                        } => {
                            let is_active =
                                env.block.time < end_time && env.block.time > start_time;
                            if is_active {
                                active_rounds.push((round_with_index.0, round_with_index.1));
                            }
                        }
                    }
                }
                // Check if any active rounds exist
                if active_rounds.len() == 0 {
                    return Err(ContractError::WhitelistNotActive {});
                }
                // Check if active rounds is greater than 1
                let active_round = active_rounds[0];
                if active_rounds.len() > 1 {
                    // Find the round which starts first
                    for round in active_rounds {
                        match round.1 {
                            Round::WhitelistAddress {
                                address,
                                start_time,
                                end_time,
                            } => {
                                if start_time.unwrap() < active_round.1.start_time() {
                                    active_round = round;
                                }
                            }
                            Round::WhitelistCollection {
                                collection_id,
                                start_time,
                                end_time,
                                mint_price,
                                per_address_limit,
                            } => {
                                if start_time < active_round.1.start_time() {
                                    active_round = round;
                                }
                            }
                        };
                    }
                }
                active_round
            }
            None => {
                return Err(ContractError::WhitelistNotActive {});
            }
        };
        // If we found an active round, check if the address is whitelisted
        match active_round.1 {
            Round::WhitelistAddress {
                address,
                start_time,
                end_time,
            } => {
                // Check if address is whitelisted
                let is_whitelisted: HasMemberResponse = deps.querier.query_wasm_smart(
                    address.clone(),
                    &WhitelistQueryMsgs::HasMember {
                        member: info.sender.clone().into_string(),
                    },
                )?;
                if !is_whitelisted.has_member {
                    return Err(ContractError::AddressNotWhitelisted {});
                }
            }
            Round::WhitelistCollection {
                collection_id,
                start_time,
                end_time,
                mint_price,
                per_address_limit,
            } => {
                // Check if address has this collection
                let nft_list_res = OnftQuerier::new(&deps.querier).owner_onf_ts(
                    collection_id,
                    info.sender.clone().into_string(),
                    None,
                )?;
                let nft_list = nft_list_res.collections;
                if nft_list.len() == 0 {
                    return Err(ContractError::AddressNotWhitelisted {});
                }
            }
        }
    }

    let collection = COLLECTION.load(deps.storage)?;

    // Check the payment
    let amount = must_pay(&info, &config.mint_denom)?;
    if amount != config.mint_price {
        return Err(ContractError::IncorrectPaymentAmount {
            expected: config.mint_price,
            sent: amount,
        });
    }

    // Check if any tokens are left
    let total_tokens_remaining = TOTAL_TOKENS_REMAINING.load(deps.storage)?;

    if total_tokens_remaining == 0 {
        return Err(ContractError::NoTokensLeftToMint {});
    }

    // Collect mintable tokens
    let mut mintable_tokens: Vec<(u32, Token)> = Vec::new();
    for item in MINTABLE_TOKENS.range(deps.storage, None, None, Order::Ascending) {
        let (key, value) = item?;

        // Add the (key, value) tuple to the vector
        mintable_tokens.push((key, value));
    }

    // Get a random token id
    let random_token = return_random_token_id(&mintable_tokens, env.clone())?;

    // Check if the address has reached the limit
    let is_mintable = check_mint_limit_for_addr(&deps, info.sender.clone(), None)?;
    if !is_mintable {
        return Err(ContractError::AddressReachedMintLimit {});
    }

    // Get the payment collector address
    let payment_collector = config.payment_collector;

    // Update storage
    // Remove the token from the mintable tokens
    MINTABLE_TOKENS.remove(deps.storage, random_token.0);

    // Decrement the total tokens remaining
    TOTAL_TOKENS_REMAINING.update(deps.storage, |mut total_tokens| -> StdResult<_> {
        total_tokens -= 1;
        Ok(total_tokens)
    })?;

    // Increment the minted tokens for the address
    let minter = MINTED_TOKENS.may_load(deps.storage, info.sender.clone())?;
    match minter {
        Some(mut minted_tokens) => {
            minted_tokens.push(random_token.clone().1);
            MINTED_TOKENS.save(deps.storage, info.sender.clone(), &minted_tokens)?;
        }
        None => {
            let minted_tokens = vec![random_token.clone().1];
            MINTED_TOKENS.save(deps.storage, info.sender.clone(), &minted_tokens)?;
        }
    }
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
    let minter = MINTED_TOKENS.may_load(deps.storage, recipient.clone())?;
    match minter {
        Some(mut minted_tokens) => {
            minted_tokens.push(token.clone().1);
            MINTED_TOKENS.save(deps.storage, recipient.clone(), &minted_tokens)?;
        }
        None => {
            let minted_tokens = vec![token.clone().1];
            MINTED_TOKENS.save(deps.storage, recipient.clone(), &minted_tokens)?;
        }
    }
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
    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.creator {
        return Err(ContractError::Unauthorized {});
    }

    let address = deps.api.addr_validate(&address)?;

    let new_config = Config {
        per_address_limit: config.per_address_limit,
        payment_collector: config.payment_collector,
        whitelist_address: Some(address.clone()),
        mint_denom: config.mint_denom,
        start_time: config.start_time,
        mint_price: config.mint_price,
        royalty_ratio: config.royalty_ratio,
        creator: config.creator,
    };

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
