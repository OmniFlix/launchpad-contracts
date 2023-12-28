use std::str::FromStr;

use crate::msg::{ExecuteMsg, QueryMsg};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Binary, Coin, CosmosMsg, Decimal, Deps, DepsMut, Env, MessageInfo, Order,
    Response, StdResult, Timestamp, Uint128, WasmMsg,
};
use cw_utils::{maybe_addr, must_pay, nonpayable};
use minter_types::{CollectionDetails, InstantiateMsg};
use omniflix_minter_factory::msg::ParamsResponse;
use omniflix_minter_factory::msg::QueryMsg::Params as QueryFactoryParams;
use round_whitelist::msg::ExecuteMsg::PrivateMint;
use whitelist_types::{MintPriceResponse, RoundWhitelistQueryMsgs};

use crate::error::ContractError;
use crate::state::{
    Config, Token, UserDetails, COLLECTION, CONFIG, MINTABLE_TOKENS, MINTED_TOKENS,
    TOTAL_TOKENS_REMAINING,
};
use crate::utils::{randomize_token_list, return_random_token_id};

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
    // Check if per address limit is 0
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
    // Check end time
    if let Some(end_time) = msg.end_time {
        if end_time < msg.start_time {
            return Err(ContractError::InvalidStartTime {});
        }
    }

    // Check royalty ratio we expect decimal number
    let royalty_ratio = Decimal::from_str(&msg.royalty_ratio)?;
    if royalty_ratio < Decimal::zero() || royalty_ratio > Decimal::one() {
        return Err(ContractError::InvalidRoyaltyRatio {});
    }
    // Check mint price
    if msg.mint_price == Uint128::new(0) {
        return Err(ContractError::InvalidMintPrice {});
    }
    let admin = maybe_addr(deps.api, msg.admin.clone())?.unwrap_or(info.sender.clone());

    let payment_collector =
        maybe_addr(deps.api, msg.payment_collector.clone())?.unwrap_or(info.sender.clone());
    let num_tokens = msg.collection_details.num_tokens;

    let config = Config {
        per_address_limit: msg.per_address_limit,
        payment_collector: payment_collector,
        start_time: msg.start_time,
        royalty_ratio: royalty_ratio,
        admin: admin,
        mint_price: Coin {
            denom: msg.mint_denom.clone(),
            amount: msg.mint_price,
        },
        whitelist_address: maybe_addr(deps.api, msg.whitelist_address.clone())?,
        end_time: msg.end_time,
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
        uri: msg.collection_details.uri,
        uri_hash: msg.collection_details.uri_hash,
        data: msg.collection_details.data,
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

    // Save mintable tokens
    for (key, value) in randomize_token_list(tokens, num_tokens, env.clone())? {
        match MINTABLE_TOKENS.save(deps.storage, key, &value) {
            Ok(_) => (),
            Err(_) => return Err(ContractError::ErrorSavingTokens {}),
        }
    }

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
        data: collection.data,
        uri: collection.uri,
        uri_hash: collection.uri_hash,
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
    }
}

pub fn execute_mint(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    _msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    // Check if any tokens are left
    let total_tokens_remaining = TOTAL_TOKENS_REMAINING.load(deps.storage)?;
    if total_tokens_remaining == 0 {
        return Err(ContractError::NoTokensLeftToMint {});
    }
    let mut user_details = MINTED_TOKENS
        .may_load(deps.storage, info.sender.clone())?
        .unwrap_or(UserDetails::new());

    // Increment total minted count
    user_details.total_minted_count += 1;
    // Check if address has reached the limit
    if user_details.total_minted_count > config.per_address_limit {
        return Err(ContractError::AddressReachedMintLimit {});
    }

    let mut mint_price = config.mint_price;

    // Collect mintable tokens
    let mut mintable_tokens: Vec<(u32, Token)> = Vec::new();
    for item in MINTABLE_TOKENS.range(deps.storage, None, None, Order::Ascending) {
        let (key, value) = item?;
        mintable_tokens.push((key, value));
    }
    // Check if public end time is determined and if it is passed
    if let Some(end_time) = config.end_time {
        if env.block.time > end_time {
            return Err(ContractError::PublicMintingEnded {});
        }
    }

    // Get a random token id
    let random_token = return_random_token_id(&mintable_tokens, env.clone())?;
    // Add the minted token to the user details
    user_details.minted_tokens.push(random_token.1.clone());
    // Check if minting is started
    let is_public = env.block.time > config.start_time;

    let mut messages: Vec<CosmosMsg> = vec![];

    if !is_public {
        // Check if any whitelist is present
        if let Some(whitelist_address) = config.whitelist_address {
            let is_active: bool = deps.querier.query_wasm_smart(
                whitelist_address.clone().into_string(),
                &RoundWhitelistQueryMsgs::IsActive {},
            )?;
            if !is_active {
                return Err(ContractError::WhitelistNotActive {});
            }
            // Check whitelist price
            let whitelist_price_response: MintPriceResponse = deps.querier.query_wasm_smart(
                whitelist_address.clone().into_string(),
                &RoundWhitelistQueryMsgs::Price {},
            )?;
            mint_price = whitelist_price_response.mint_price;
            // Check if member is whitelisted
            let is_whitelisted: bool = deps.querier.query_wasm_smart(
                whitelist_address.clone().into_string(),
                &RoundWhitelistQueryMsgs::IsMember {
                    address: info.sender.clone().into_string(),
                },
            )?;
            if !is_whitelisted {
                return Err(ContractError::AddressNotWhitelisted {});
            }
            messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: whitelist_address.into_string(),
                msg: to_json_binary(&PrivateMint {
                    admin: config.admin.into_string(),
                    minter: info.sender.clone().into_string(),
                })?,
                funds: vec![],
            }));
        } else {
            return Err(ContractError::MintingNotStarted {
                start_time: config.start_time,
                current_time: env.block.time,
            });
        };
    }

    // Check the payment
    let amount = must_pay(&info, &mint_price.denom)?;
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
    // Update storage
    // Remove the token from the mintable tokens
    MINTABLE_TOKENS.remove(deps.storage, random_token.0);
    // Decrement the total tokens remaining
    TOTAL_TOKENS_REMAINING.update(deps.storage, |mut total_tokens| -> StdResult<_> {
        total_tokens -= 1;
        Ok(total_tokens)
    })?;
    // Save the user details
    MINTED_TOKENS.save(deps.storage, info.sender.clone(), &user_details)?;
    let token_id = random_token.1.token_id;
    // Generate the metadata
    let metadata = Metadata {
        name: format!("{} # {}", collection.name, token_id),
        description: collection.description,
        media_uri: format!("{}/{}", collection.base_uri, token_id),
        preview_uri: collection.preview_uri,
        uri_hash: collection.uri_hash,
    };

    // Create the mint message
    let mint_msg: CosmosMsg = MsgMintOnft {
        data: "".to_string(),
        id: format!("{}{}", collection.id, token_id),
        metadata: Some(metadata.clone()),
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
            denom: mint_price.denom,
            amount: mint_price.amount,
        }],
    })
    .into();

    messages.push(mint_msg.clone());
    messages.push(bank_msg.clone());

    let res = Response::new()
        .add_messages(messages)
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
    if info.sender != config.admin {
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
    user_details.minted_tokens.push(token.1.clone());
    user_details.total_minted_count += 1;
    // Save details
    MINTED_TOKENS.save(deps.storage, recipient.clone(), &user_details)?;

    let denom_id = token.1.token_id;

    // Generate the metadata
    let metadata = Metadata {
        name: format!("{} # {}", collection.name, denom_id),
        description: collection.description,
        media_uri: format!("{}/{}", collection.preview_uri, denom_id),
        preview_uri: collection.preview_uri,
        uri_hash: collection.uri_hash,
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
    _env: Env,
    info: MessageInfo,
    ratio: String,
) -> Result<Response, ContractError> {
    // Check if sender is admin
    let mut config = CONFIG.load(deps.storage)?;
    if info.sender != config.admin {
        return Err(ContractError::Unauthorized {});
    }
    let ratio = Decimal::from_str(&ratio)?; // Check if ratio is decimal number
    if ratio < Decimal::zero() || ratio > Decimal::one() {
        return Err(ContractError::InvalidRoyaltyRatio {});
    }
    config.royalty_ratio = ratio;

    CONFIG.save(deps.storage, &config)?;

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
    let mut config = CONFIG.load(deps.storage)?;
    if info.sender != config.admin {
        return Err(ContractError::Unauthorized {});
    }
    // Check if trading has started
    if env.block.time > config.start_time {
        return Err(ContractError::MintingAlreadyStarted {});
    }
    // Check if mint price is valid
    if mint_price == Uint128::new(0) {
        return Err(ContractError::InvalidMintPrice {});
    }
    config.mint_price.amount = mint_price;

    CONFIG.save(deps.storage, &config)?;

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
) -> Result<UserDetails, ContractError> {
    let address = deps.api.addr_validate(&address)?;
    let minted_tokens = MINTED_TOKENS.load(deps.storage, address)?;
    Ok(minted_tokens)
}

fn query_total_tokens(deps: Deps, _env: Env) -> Result<u32, ContractError> {
    let total_tokens = TOTAL_TOKENS_REMAINING.load(deps.storage)?;
    Ok(total_tokens)
}
