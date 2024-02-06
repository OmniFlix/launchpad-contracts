use std::str::FromStr;

//use crate::msg::ExecuteMsg;
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Addr, Binary, Coin, CosmosMsg, Decimal, Deps, DepsMut, Env, MessageInfo,
    Response, StdResult, Timestamp, Uint128, WasmMsg,
};
use cw_utils::{maybe_addr, must_pay, nonpayable};
use minter_types::{
    generate_mint_message, CollectionDetails, Config, PauseState, Token, UserDetails,
};
use open_edition_minter_types::QueryMsg;

use crate::error::ContractError;
use crate::msg::ExecuteMsg;
use crate::state::{
    EditionParams, MintedTokens, CURRENT_EDITION, EDITIONS, LAST_MINTED_TOKEN_ID, MINTED_COUNT,
    MINTED_TOKENS_KEY,
};
use cw2::set_contract_version;
use omniflix_open_edition_minter_factory::msg::{
    OpenEditionMinterCreateMsg, ParamsResponse, QueryMsg as OpenEditionMinterFactoryQueryMsg,
};
use omniflix_round_whitelist::msg::ExecuteMsg as RoundWhitelistExecuteMsg;
use omniflix_std::types::omniflix::onft::v1beta1::{MsgCreateDenom, OnftQuerier, WeightedAddress};
use whitelist_types::{
    check_if_address_is_member, check_if_whitelist_is_active, check_whitelist_price,
};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:omniflix-minter-open-edition-minter";
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

const PAUSED_KEY: &str = "paused";
const PAUSERS_KEY: &str = "pausers";

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: OpenEditionMinterCreateMsg,
) -> Result<Response, ContractError> {
    // Query denom creation fee
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    // Query factory params of instantiator
    // If the instantiator is not our factory then we wont be able to parse the response
    let _factory_params: ParamsResponse = deps.querier.query_wasm_smart(
        info.sender.clone().into_string(),
        &OpenEditionMinterFactoryQueryMsg::Params {},
    )?;

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
    // Check if token limit is 0
    if let Some(token_limit) = msg.init.token_limit {
        if token_limit == 0 {
            return Err(ContractError::InvalidNumTokens {});
        }
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
    let royalty_ratio = Decimal::from_str(&msg.init.royalty_ratio)?;
    if royalty_ratio < Decimal::zero() || royalty_ratio > Decimal::one() {
        return Err(ContractError::InvalidRoyaltyRatio {});
    }
    // Check if whitelist already active
    if let Some(whitelist_address) = msg.init.whitelist_address.clone() {
        let is_active: bool = check_if_whitelist_is_active(
            &deps.api.addr_validate(&whitelist_address)?,
            deps.as_ref(),
        )?;
        if is_active {
            return Err(ContractError::WhitelistAlreadyActive {});
        }
    }

    let admin = deps.api.addr_validate(&msg.init.admin)?;

    let payment_collector =
        maybe_addr(deps.api, msg.init.payment_collector.clone())?.unwrap_or(info.sender.clone());

    let config = Config {
        per_address_limit: msg.init.per_address_limit,
        payment_collector,
        start_time: msg.init.start_time,
        royalty_ratio,
        admin: admin.clone(),
        mint_price: msg.init.mint_price,
        whitelist_address: maybe_addr(deps.api, msg.init.whitelist_address.clone())?,
        end_time: msg.init.end_time,
        token_limit: msg.init.token_limit,
    };
    // Set the pause state
    let pause_state = PauseState::new(PAUSED_KEY, PAUSERS_KEY)?;
    pause_state.set_pausers(deps.storage, info.sender.clone(), vec![admin.clone()])?;

    let collection = CollectionDetails {
        name: msg.collection_details.name,
        description: msg.collection_details.description,
        preview_uri: msg.collection_details.preview_uri,
        schema: msg.collection_details.schema,
        symbol: msg.collection_details.symbol,
        id: msg.collection_details.id,
        extensible: msg.collection_details.extensible,
        nsfw: msg.collection_details.nsfw,
        base_uri: msg.collection_details.base_uri,
        uri: msg.collection_details.uri,
        uri_hash: msg.collection_details.uri_hash,
        data: msg.collection_details.data,
        token_name: msg.collection_details.token_name,
        transferable: msg.collection_details.transferable,
        royalty_receivers: msg.collection_details.royalty_receivers,
    };

    // Save the collection as edition 1
    let edition_params = EditionParams {
        config: config.clone(),
        collection: collection.clone(),
    };

    EDITIONS.save(deps.storage, 1, &edition_params)?;
    CURRENT_EDITION.save(deps.storage, &1)?;
    MINTED_COUNT.save(deps.storage, 1, &0)?;
    LAST_MINTED_TOKEN_ID.save(deps.storage, &0)?;

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
        royalty_receivers: collection
            .royalty_receivers
            .unwrap_or(vec![WeightedAddress {
                address: admin.clone().into_string(),
                weight: Decimal::one().to_string(),
            }]),
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
        ExecuteMsg::Mint {} => execute_mint(deps, env, info),
        ExecuteMsg::MintAdmin { recipient } => execute_mint_admin(deps, env, info, recipient),
        ExecuteMsg::UpdateRoyaltyRatio { ratio } => {
            execute_update_royalty_ratio(deps, env, info, ratio)
        }
        ExecuteMsg::UpdateMintPrice { mint_price } => {
            execute_update_mint_price(deps, env, info, mint_price)
        }
        ExecuteMsg::UpdateWhitelistAddress { address } => {
            execute_update_whitelist_address(deps, env, info, address)
        }
        ExecuteMsg::Pause {} => execute_pause(deps, env, info),
        ExecuteMsg::Unpause {} => execute_unpause(deps, env, info),
        ExecuteMsg::SetPausers { pausers } => execute_set_pausers(deps, env, info, pausers),
        ExecuteMsg::NewEdition {
            whitelist_address,
            token_limit,
            start_time,
            end_time,
            mint_price,
            royalty_ratio,
            token_name,
            description,
            base_uri,
            preview_uri,
            uri_hash,
            transferable,
            extensible,
            nsfw,
            data,
        } => execute_new_edition(
            deps,
            env,
            info,
            whitelist_address,
            token_limit,
            start_time,
            end_time,
            mint_price,
            royalty_ratio,
            token_name,
            description,
            base_uri,
            preview_uri,
            uri_hash,
            transferable,
            extensible,
            nsfw,
            data,
        ),
    }
}

pub fn execute_mint(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    let pause_state = PauseState::new(PAUSED_KEY, PAUSERS_KEY)?;
    pause_state.error_if_paused(deps.storage)?;

    // Find the latest edition
    let edition_number = CURRENT_EDITION.load(deps.storage)?;
    let edition_params = EDITIONS.load(deps.storage, edition_number)?;

    let config = edition_params.config;
    let collection = edition_params.collection;
    // Check if any token limit set and if it is reached
    if let Some(token_limit) = config.token_limit {
        if MINTED_COUNT.load(deps.storage, edition_number).unwrap_or(0) >= token_limit {
            return Err(ContractError::NoTokensLeftToMint {});
        }
    }

    // Check if end time is determined and if it is passed
    if let Some(end_time) = config.end_time {
        if env.block.time > end_time {
            return Err(ContractError::PublicMintingEnded {});
        }
    };
    let minted_tokens = MintedTokens::new(MINTED_TOKENS_KEY);
    let mut user_details = minted_tokens
        .load(deps.storage, edition_number, info.sender.clone())
        .unwrap_or_default();

    let last_token_id = LAST_MINTED_TOKEN_ID.load(deps.storage)?;
    let token_id = last_token_id + 1;
    LAST_MINTED_TOKEN_ID.save(deps.storage, &token_id)?;

    let mut mint_price = config.mint_price;
    // Check if minting is started

    let is_public = env.block.time >= config.start_time;

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
            let is_member =
                check_if_address_is_member(&info.sender, &whitelist_address, deps.as_ref())?;
            if !is_member {
                return Err(ContractError::AddressNotWhitelisted {});
            }
            messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: whitelist_address.into_string(),
                msg: to_json_binary(&RoundWhitelistExecuteMsg::PrivateMint {
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
        if user_details.public_mint_count > config.per_address_limit {
            return Err(ContractError::AddressReachedMintLimit {});
        }
    }
    // Increment total minted count
    user_details.total_minted_count += 1;

    user_details.minted_tokens.push(Token {
        token_id: token_id.to_string(),
    });
    // Save the user details
    minted_tokens.save(
        deps.storage,
        edition_number,
        info.sender.clone(),
        &user_details,
    );

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
    //let collection = COLLECTION.load(deps.storage)?;

    let mut minted_count = MINTED_COUNT.load(deps.storage, edition_number)?;
    minted_count += 1;
    MINTED_COUNT.save(deps.storage, edition_number, &minted_count)?;

    let mint_msg: CosmosMsg = generate_mint_message(
        &collection,
        config.royalty_ratio,
        &info.sender,
        &env.contract.address,
        true,
        token_id.to_string(),
    )
    .into();

    // Create the Bank send message
    let bank_msg: CosmosMsg = CosmosMsg::Bank(cosmwasm_std::BankMsg::Send {
        to_address: payment_collector.into_string(),
        amount: vec![Coin {
            denom: mint_price.denom,
            amount: mint_price.amount,
        }],
    });

    messages.push(mint_msg.clone());
    messages.push(bank_msg.clone());

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
) -> Result<Response, ContractError> {
    nonpayable(&info)?;

    let edition_number = CURRENT_EDITION.load(deps.storage)?;
    let edition_params = EDITIONS.load(deps.storage, edition_number)?;

    let config = edition_params.config;
    let collection = edition_params.collection;

    let pause_state = PauseState::new(PAUSED_KEY, PAUSERS_KEY)?;
    pause_state.error_if_paused(deps.storage)?;

    // Check if sender is admin
    if info.sender != config.admin {
        return Err(ContractError::Unauthorized {});
    }
    let recipient = deps.api.addr_validate(&recipient)?;
    // We are not checking token limit nor end time here because this is the admin minting
    let last_token_id = LAST_MINTED_TOKEN_ID.load(deps.storage)?;
    let token_id = last_token_id + 1;

    let minted_tokens = MintedTokens::new(MINTED_TOKENS_KEY);

    let mut user_details = minted_tokens
        .load(deps.storage, edition_number, recipient.clone())
        .unwrap_or_default();

    user_details.total_minted_count += 1;
    user_details.minted_tokens.push(Token {
        token_id: token_id.to_string(),
    });
    // Save the user details
    minted_tokens.save(
        deps.storage,
        edition_number,
        recipient.clone(),
        &user_details,
    );

    let mint_msg: CosmosMsg = generate_mint_message(
        &collection,
        config.royalty_ratio,
        &recipient,
        &env.contract.address,
        true,
        token_id.to_string(),
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
    let edition_number = CURRENT_EDITION.load(deps.storage)?;
    let mut edition_params = EDITIONS.load(deps.storage, edition_number)?;
    if info.sender != edition_params.clone().config.admin {
        return Err(ContractError::Unauthorized {});
    }
    edition_params.config.token_limit = Some(0);
    EDITIONS.save(deps.storage, edition_number, &edition_params)?;
    let res = Response::new().add_attribute("action", "burn_remaining_tokens");
    Ok(res)
}

pub fn execute_update_royalty_ratio(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    ratio: String,
) -> Result<Response, ContractError> {
    let edition_number = CURRENT_EDITION.load(deps.storage)?;
    let mut edition_params = EDITIONS.load(deps.storage, edition_number)?;

    // Check if sender is admin
    if info.sender != edition_params.config.admin {
        return Err(ContractError::Unauthorized {});
    }
    // Check if ratio is decimal number
    let ratio = Decimal::from_str(&ratio)?;

    if ratio < Decimal::zero() || ratio > Decimal::one() {
        return Err(ContractError::InvalidRoyaltyRatio {});
    }
    edition_params.config.royalty_ratio = ratio;

    EDITIONS.save(deps.storage, edition_number, &edition_params)?;

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
    let edition_number = CURRENT_EDITION.load(deps.storage)?;
    let mut edition_params = EDITIONS.load(deps.storage, edition_number)?;

    // Check if sender is admin
    if info.sender != edition_params.config.admin {
        return Err(ContractError::Unauthorized {});
    }
    edition_params.config.mint_price = mint_price.clone();

    EDITIONS.save(deps.storage, edition_number, &edition_params)?;

    let res = Response::new()
        .add_attribute("action", "update_mint_price")
        .add_attribute("mint_price_denom", mint_price.denom.to_string())
        .add_attribute("mint_price_amount", mint_price.amount.to_string());
    Ok(res)
}

pub fn execute_update_whitelist_address(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    address: String,
) -> Result<Response, ContractError> {
    // Check if sender is admin
    let edition_number = CURRENT_EDITION.load(deps.storage)?;
    let mut edition_params = EDITIONS.load(deps.storage, edition_number)?;
    if info.sender != edition_params.config.admin {
        return Err(ContractError::Unauthorized {});
    }
    let whitelist_address = edition_params.config.whitelist_address.clone();
    // Check if whitelist already active
    match whitelist_address {
        Some(whitelist_address) => {
            let is_active: bool = check_if_whitelist_is_active(&whitelist_address, deps.as_ref())?;
            if is_active {
                return Err(ContractError::WhitelistAlreadyActive {});
            }
        }
        None => {}
    }

    let address = deps.api.addr_validate(&address)?;
    let is_active: bool = check_if_whitelist_is_active(&address, deps.as_ref())?;
    if is_active {
        return Err(ContractError::WhitelistAlreadyActive {});
    }
    edition_params.config.whitelist_address = Some(address.clone());
    EDITIONS.save(deps.storage, edition_number, &edition_params)?;

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

pub fn execute_new_edition(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    whitelist_address: Option<String>,
    token_limit: Option<u32>,
    start_time: Timestamp,
    end_time: Option<Timestamp>,
    mint_price: Coin,
    royalty_ratio: String,
    token_name: String,
    description: String,
    base_uri: String,
    preview_uri: String,
    uri_hash: String,
    transferable: bool,
    extensible: bool,
    nsfw: bool,
    data: String,
) -> Result<Response, ContractError> {
    // Check if sender is admin
    let current_edition_number = CURRENT_EDITION.load(deps.storage)?;
    let current_edition_params = EDITIONS.load(deps.storage, current_edition_number)?;
    if info.sender != current_edition_params.config.admin {
        return Err(ContractError::Unauthorized {});
    }
    // Check if token limit is 0
    if let Some(token_limit) = token_limit {
        if token_limit == 0 {
            return Err(ContractError::InvalidNumTokens {});
        }
    }
    // Check start time
    if start_time < env.block.time {
        return Err(ContractError::InvalidStartTime {});
    }
    // Check end time
    if let Some(end_time) = end_time {
        if end_time < start_time {
            return Err(ContractError::InvalidEndTime {});
        }
    }
    // Check royalty ratio we expect decimal number
    let royalty_ratio = Decimal::from_str(&royalty_ratio)?;
    if royalty_ratio < Decimal::zero() || royalty_ratio > Decimal::one() {
        return Err(ContractError::InvalidRoyaltyRatio {});
    }
    // Check if whitelist already active
    if let Some(whitelist_address) = whitelist_address.clone() {
        let is_active: bool = check_if_whitelist_is_active(
            &deps.api.addr_validate(&whitelist_address)?,
            deps.as_ref(),
        )?;
        if is_active {
            return Err(ContractError::WhitelistAlreadyActive {});
        }
    }

    let config = Config {
        per_address_limit: current_edition_params.config.per_address_limit,
        payment_collector: current_edition_params.config.payment_collector,
        start_time,
        royalty_ratio,
        admin: current_edition_params.config.admin,
        mint_price,
        whitelist_address: maybe_addr(deps.api, whitelist_address)?,
        end_time,
        token_limit,
    };
    let collection = CollectionDetails {
        name: current_edition_params.collection.name,
        description,
        preview_uri,
        schema: current_edition_params.collection.schema,
        symbol: current_edition_params.collection.symbol,
        id: current_edition_params.collection.id,
        extensible,
        nsfw,
        base_uri,
        uri: current_edition_params.collection.uri,
        uri_hash,
        data: data.clone(),
        token_name,
        transferable,
        royalty_receivers: current_edition_params.collection.royalty_receivers,
    };
    let edition_params = EditionParams { config, collection };
    let new_edition_number = current_edition_number + 1;
    EDITIONS.save(deps.storage, new_edition_number, &edition_params)?;
    CURRENT_EDITION.save(deps.storage, &new_edition_number)?;
    MINTED_COUNT.save(deps.storage, new_edition_number, &0)?;

    let res = Response::new()
        .add_attribute("action", "new_edition")
        .add_attribute("edition_number", new_edition_number.to_string());

    Ok(res)
}

// Implement Queries
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Collection { edition } => to_json_binary(&query_collection(deps, env, edition)?),
        QueryMsg::Config { edition } => to_json_binary(&query_config(deps, env, edition)?),
        QueryMsg::MintedTokens { address, edition } => {
            to_json_binary(&query_minted_tokens(deps, env, address, edition)?)
        }
        QueryMsg::TotalMintedCount { edition } => {
            to_json_binary(&query_total_tokens_minted(deps, env, edition)?)
        }
        QueryMsg::TokensRemaining { edition } => {
            to_json_binary(&query_tokens_remaining(deps, env, edition)?)
        }
        QueryMsg::IsPaused {} => to_json_binary(&query_is_paused(deps, env)?),
        QueryMsg::Pausers {} => to_json_binary(&query_pausers(deps, env)?),
    }
}

fn query_collection(
    deps: Deps,
    _env: Env,
    edition: Option<u32>,
) -> Result<CollectionDetails, ContractError> {
    let edition = edition.unwrap_or(CURRENT_EDITION.load(deps.storage)?);
    let edition_params = EDITIONS.load(deps.storage, edition)?;
    let collection = edition_params.collection;
    Ok(collection)
}

fn query_config(deps: Deps, _env: Env, edition: Option<u32>) -> Result<Config, ContractError> {
    let edition = edition.unwrap_or(CURRENT_EDITION.load(deps.storage)?);
    let edition_params = EDITIONS.load(deps.storage, edition)?;
    let config = edition_params.config;
    Ok(config)
}

fn query_minted_tokens(
    deps: Deps,
    _env: Env,
    address: String,
    edition: Option<u32>,
) -> Result<UserDetails, ContractError> {
    let address = deps.api.addr_validate(&address)?;
    let edition = edition.unwrap_or(CURRENT_EDITION.load(deps.storage)?);
    let minted_tokens = MintedTokens::new(MINTED_TOKENS_KEY);
    let user_details = minted_tokens
        .load(deps.storage, edition, address)
        .unwrap_or_default();
    Ok(user_details)
}

fn query_total_tokens_minted(
    deps: Deps,
    _env: Env,
    edition: Option<u32>,
) -> Result<u32, ContractError> {
    let edition = edition.unwrap_or(CURRENT_EDITION.load(deps.storage)?);
    let minted_count = MINTED_COUNT.load(deps.storage, edition).unwrap_or(0);
    Ok(minted_count)
}

fn query_tokens_remaining(
    deps: Deps,
    _env: Env,
    edition: Option<u32>,
) -> Result<u32, ContractError> {
    let edition = edition.unwrap_or(CURRENT_EDITION.load(deps.storage)?);
    let edition_params = EDITIONS.load(deps.storage, edition)?;
    let config = edition_params.config;
    let minted_count = MINTED_COUNT.load(deps.storage, edition).unwrap_or(0);
    let tokens_remaining = config.token_limit.unwrap_or(u32::MAX) - minted_count;
    Ok(tokens_remaining)
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
