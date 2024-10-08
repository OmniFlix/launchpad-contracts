#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Addr, BankMsg, Binary, Coin, CosmosMsg, Decimal, Deps, DepsMut, Env,
    MessageInfo, Response, StdResult, WasmMsg,
};
use cw_utils::{may_pay, maybe_addr, must_pay, nonpayable};
use minter_types::collection_details::{update_collection_details, CollectionDetails};
use minter_types::config::Config;
use minter_types::msg::{MintHistoryResponse, QueryMsg as BaseMinterQueryMsg};
use minter_types::token_details::{Token, TokenDetails};
use minter_types::types::{AuthDetails, UserDetails};
use minter_types::utils::{
    check_collection_creation_fee, generate_create_denom_msg, generate_oem_mint_message,
    generate_update_denom_msg,
};
use std::str::FromStr;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, OEMQueryExtension};
use crate::state::{
    last_token_id, AUTH_DETAILS, COLLECTION, CONFIG, MINTED_COUNT, TOKEN_DETAILS,
    USER_MINTING_DETAILS,
};
use cw2::set_contract_version;
use omniflix_open_edition_minter_factory::msg::{
    OpenEditionMinterCreateMsg, ParamsResponse, QueryMsg as OpenEditionMinterFactoryQueryMsg,
};
use omniflix_round_whitelist::msg::ExecuteMsg as RoundWhitelistExecuteMsg;
use omniflix_std::types::omniflix::onft::v1beta1::{MsgPurgeDenom, WeightedAddress};
use pauser::PauseState;
use whitelist_types::{
    check_if_address_is_member, check_if_whitelist_is_active, check_whitelist_price,
};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:omniflix-minter-open-edition-minter";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: OpenEditionMinterCreateMsg,
) -> Result<Response, ContractError> {
    // Set the contract version
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    // Query factory parameters of the instantiator
    // If the instantiator is not our factory, we won't be able to parse the response
    let _factory_params: ParamsResponse = deps.querier.query_wasm_smart(
        info.sender.clone().into_string(),
        &OpenEditionMinterFactoryQueryMsg::Params {},
    )?;

    // Retrieve the collection creation fee
    let collection_creation_fee: Coin = check_collection_creation_fee(deps.as_ref().querier)?;

    // Validate token details
    if msg.token_details.is_none() {
        return Err(ContractError::InvalidTokenDetails {});
    }
    let token_details = msg.token_details.clone().unwrap();

    // Extract other necessary information
    let collection_details = msg.collection_details.clone();

    if msg.init.is_none() {
        return Err(ContractError::InitMissing {});
    }
    let init = msg.init.clone().unwrap();

    // Initialize configuration
    let config = Config {
        per_address_limit: init.per_address_limit,
        start_time: init.start_time,
        mint_price: init.mint_price,
        whitelist_address: maybe_addr(deps.api, init.whitelist_address.clone())?,
        end_time: init.end_time,
        num_tokens: init.num_tokens,
    };

    // Check integrity of token details and configuration
    token_details.clone().check_integrity()?;
    config.clone().check_integrity(env.block.time)?;

    // Validate payment amount
    let amount = must_pay(&info, &collection_creation_fee.denom)?;
    // Exact amount must be paid
    if amount != collection_creation_fee.amount {
        return Err(ContractError::InvalidCreationFee {
            expected: vec![collection_creation_fee.clone()],
            sent: info.funds,
        });
    }

    // Check if whitelist is already active
    if let Some(whitelist_address) = init.whitelist_address.clone() {
        let is_active = check_if_whitelist_is_active(
            &deps.api.addr_validate(&whitelist_address)?,
            deps.as_ref(),
        )?;
        if is_active {
            return Err(ContractError::WhitelistAlreadyActive {});
        }
    }
    let auth_details = msg.auth_details.clone();
    auth_details.validate(&deps.as_ref())?;
    CONFIG.save(deps.storage, &config)?;
    MINTED_COUNT.save(deps.storage, &0)?;
    AUTH_DETAILS.save(deps.storage, &auth_details)?;

    // Initialize pause state and set admin as pauser
    let pause_state = PauseState::new()?;
    pause_state.set_pausers(
        deps.storage,
        info.sender.clone(),
        vec![auth_details.admin.clone()],
    )?;

    // Save collection and token details
    COLLECTION.save(deps.storage, &collection_details)?;
    TOKEN_DETAILS.save(deps.storage, &token_details)?;

    // Prepare and send the create denom message
    let creation_fee = Coin {
        denom: collection_creation_fee.denom,
        amount: collection_creation_fee.amount,
    };

    let collection_creation_msg: CosmosMsg = generate_create_denom_msg(
        &collection_details,
        env.contract.address.clone(),
        creation_fee,
        auth_details.payment_collector.clone(),
    )?
    .into();

    // Prepare the response with attributes
    let res = Response::new()
        .add_message(collection_creation_msg)
        .add_attribute("action", "instantiate")
        .add_attribute("collection_id", collection_details.id)
        .add_attribute("contract_address", env.contract.address.clone())
        .add_attribute("admin", auth_details.admin.to_string())
        .add_attribute(
            "payment_collector",
            auth_details.payment_collector.to_string(),
        )
        .add_attribute("mint_price", config.mint_price.to_string())
        .add_attribute("start_time", config.start_time.to_string());

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
        ExecuteMsg::BurnRemainingTokens {} => execute_burn_remaining_tokens(deps, env, info),
    }
}

pub fn execute_mint(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    // Ensure the contract is not paused
    let pause_state = PauseState::new()?;
    pause_state.error_if_paused(deps.storage)?;

    // Load contract configuration and authorization details
    let config = CONFIG.load(deps.storage)?;
    let auth_details = AUTH_DETAILS.load(deps.storage)?;

    // Check if the number of tokens has reached the limit, if set
    if let Some(num_tokens) = config.num_tokens {
        if MINTED_COUNT.load(deps.storage)? >= num_tokens {
            return Err(ContractError::NoTokensLeftToMint {});
        }
    }

    // Check if the minting period has ended, if specified
    if let Some(end_time) = config.end_time {
        if env.block.time > end_time {
            return Err(ContractError::PublicMintingEnded {});
        }
    }

    // Generate a new token ID
    let token_id = last_token_id(deps.storage) + 1;

    // Load or initialize user minting details
    let mut user_details = USER_MINTING_DETAILS
        .may_load(deps.storage, info.sender.clone())?
        .unwrap_or_default();

    // Increment the total minted count for the user
    user_details.total_minted_count += 1;

    // Update user's minted tokens list
    user_details.minted_tokens.push(Token {
        token_id: token_id.to_string(),
    });

    let mut mint_price = config.mint_price;

    // Check if minting is started
    let is_public = env.block.time >= config.start_time;

    let mut messages: Vec<CosmosMsg> = vec![];

    // If minting is not public, handle private minting
    if !is_public {
        // Check if any whitelist is active
        if let Some(whitelist_address) = config.whitelist_address {
            let whitelist_price = check_whitelist_price(&whitelist_address, deps.as_ref())
                .map_err(|_| ContractError::WhitelistNotActive {})?;
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

            // If member is whitelisted, execute private mint
            let execute_msg = RoundWhitelistExecuteMsg::PrivateMint {
                collector: info.sender.clone().into_string(),
            };
            let msg = CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: whitelist_address.into_string(),
                msg: to_json_binary(&execute_msg)?,
                funds: vec![],
            });
            messages.push(msg);
        } else {
            // Error if minting is not started
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
    // Save updated user details
    USER_MINTING_DETAILS.save(deps.storage, info.sender.clone(), &user_details)?;

    // Validate payment
    let amount = may_pay(&info, &mint_price.denom)?;
    if amount != mint_price.amount {
        return Err(ContractError::IncorrectPaymentAmount {
            expected: mint_price.amount,
            sent: amount,
        });
    }

    // Get the payment collector address
    let payment_collector = auth_details.payment_collector;

    // Load collection and token details
    let collection = COLLECTION.load(deps.storage)?;
    let token_details = TOKEN_DETAILS.load(deps.storage)?;

    // Increment total minted count
    MINTED_COUNT.update(deps.storage, |mut total_tokens| -> StdResult<_> {
        total_tokens += 1;
        Ok(total_tokens)
    })?;

    // Create the mint message
    let mint_msg: CosmosMsg = generate_oem_mint_message(
        &collection,
        &token_details,
        token_id.to_string(),
        env.contract.address.clone(),
        info.sender.clone(),
    )?
    .into();
    messages.push(mint_msg);

    // Create the Bank send message if mint_price is non-zero
    if !mint_price.amount.is_zero() {
        let bank_msg = CosmosMsg::Bank(BankMsg::Send {
            to_address: payment_collector.into_string(),
            amount: vec![Coin {
                denom: mint_price.denom,
                amount: mint_price.amount,
            }],
        });
        messages.push(bank_msg);
    }

    // Prepare response with attributes
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
    // Ensure the function is not payable
    nonpayable(&info)?;

    // Load necessary contract data
    let config = CONFIG.load(deps.storage)?;
    let pause_state = PauseState::new()?;
    pause_state.error_if_paused(deps.storage)?;
    let collection = COLLECTION.load(deps.storage)?;
    let token_details = TOKEN_DETAILS.load(deps.storage)?;
    let auth_details = AUTH_DETAILS.load(deps.storage)?;

    // Check if the sender is authorized as admin
    if info.sender != auth_details.admin {
        return Err(ContractError::Unauthorized {});
    }

    // Check if the number of tokens has reached the limit, if set
    if let Some(num_tokens) = config.num_tokens {
        if MINTED_COUNT.load(deps.storage)? >= num_tokens {
            return Err(ContractError::NoTokensLeftToMint {});
        }
    }

    // Check if the minting period has ended, if specified
    if let Some(end_time) = config.end_time {
        if env.block.time > end_time {
            return Err(ContractError::PublicMintingEnded {});
        }
    }

    // Validate recipient address
    let recipient = deps.api.addr_validate(&recipient)?;

    // Generate a new token ID
    let token_id = last_token_id(deps.storage) + 1;

    // Load or initialize user minting details
    let mut user_details = USER_MINTING_DETAILS
        .may_load(deps.storage, recipient.clone())?
        .unwrap_or_default();

    // Increment the total minted count for the user
    user_details.total_minted_count += 1;

    // Update user's minted tokens list
    user_details.minted_tokens.push(Token {
        token_id: token_id.to_string(),
    });

    // Save updated user details
    USER_MINTING_DETAILS.save(deps.storage, recipient.clone(), &user_details)?;

    // Increment total minted count
    MINTED_COUNT.update(deps.storage, |mut total_tokens| -> StdResult<_> {
        total_tokens += 1;
        Ok(total_tokens)
    })?;

    // Create the mint message
    let mint_msg: CosmosMsg = generate_oem_mint_message(
        &collection,
        &token_details,
        token_id.to_string(),
        env.contract.address,
        recipient,
    )?
    .into();

    // Prepare response with attributes
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
    // Check if admin
    if info.sender != auth_details.admin {
        return Err(ContractError::Unauthorized {});
    }
    // We cannot burn open edition minter but we can set token limit to 0
    let mut config = CONFIG.load(deps.storage)?;

    config.num_tokens = Some(0);
    CONFIG.save(deps.storage, &config)?;

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
    // Check if admin
    if info.sender != auth_details.admin {
        return Err(ContractError::Unauthorized {});
    }
    // Check if ratio is decimal number
    let ratio = Decimal::from_str(&ratio)?;
    let mut token_details = TOKEN_DETAILS.load(deps.storage)?;
    token_details.royalty_ratio = ratio;
    // Check if token details are valid
    token_details.check_integrity()?;
    TOKEN_DETAILS.save(deps.storage, &token_details)?;

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
    let auth_details = AUTH_DETAILS.load(deps.storage)?;
    // Check if admin
    if info.sender != auth_details.admin {
        return Err(ContractError::Unauthorized {});
    }
    config.mint_price = mint_price.clone();

    CONFIG.save(deps.storage, &config)?;

    let res = Response::new()
        .add_attribute("action", "update_mint_price")
        .add_attribute("mint_price_amount", mint_price.amount.to_string())
        .add_attribute("denom", mint_price.denom);
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
    // Check if admin
    if info.sender != auth_details.admin {
        return Err(ContractError::Unauthorized {});
    }
    // Current whitelist can not be active if we are updating it
    if let Some(whitelist_address) = config.whitelist_address.clone() {
        let is_active = check_if_whitelist_is_active(&whitelist_address, deps.as_ref())?;
        if is_active {
            return Err(ContractError::WhitelistAlreadyActive {});
        }
    }
    let address = deps.api.addr_validate(&address)?;
    // Proposed whitelist can not be active if we are updating it
    let is_active = check_if_whitelist_is_active(&address, deps.as_ref())?;
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

pub fn execute_update_admin(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    admin: String,
) -> Result<Response, ContractError> {
    // Check if sender is admin
    let mut auth_details = AUTH_DETAILS.load(deps.storage)?;
    // Check if admin
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
    // Check if admin
    if info.sender != auth_details.admin {
        return Err(ContractError::Unauthorized {});
    }
    let new_payment_collector = deps.api.addr_validate(&payment_collector)?;
    auth_details.payment_collector = new_payment_collector.clone();
    AUTH_DETAILS.save(deps.storage, &auth_details)?;

    let res = Response::new()
        .add_attribute("action", "update_payment_collector")
        .add_attribute("payment_collector", new_payment_collector.to_string());
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
    // Check if admin
    if info.sender != auth_details.admin {
        return Err(ContractError::Unauthorized {});
    }
    let new_collection_details =
        update_collection_details(&collection_details, None, None, None, Some(receivers));

    COLLECTION.save(deps.storage, &new_collection_details)?;

    let update_msg: CosmosMsg = generate_update_denom_msg(
        &new_collection_details,
        auth_details.payment_collector,
        env.contract.address,
    )?
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
    collection_name: Option<String>,
    description: Option<String>,
    preview_uri: Option<String>,
) -> Result<Response, ContractError> {
    // Check if sender is admin
    let collection_details = COLLECTION.load(deps.storage)?;
    let auth_details = AUTH_DETAILS.load(deps.storage)?;
    // Check if admin
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

    let update_msg: CosmosMsg = generate_update_denom_msg(
        &new_collection_details,
        auth_details.payment_collector,
        env.contract.address,
    )?
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
    let auth_details = AUTH_DETAILS.load(deps.storage)?;
    // Check if admin
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
    msg: BaseMinterQueryMsg<OEMQueryExtension>,
) -> StdResult<Binary> {
    match msg {
        BaseMinterQueryMsg::MintHistory { address } => {
            to_json_binary(&query_mint_history(deps, env, address)?)
        }
        BaseMinterQueryMsg::Collection {} => to_json_binary(&query_collection(deps, env)?),
        BaseMinterQueryMsg::TokenDetails {} => to_json_binary(&query_token_details(deps, env)?),
        BaseMinterQueryMsg::Config {} => to_json_binary(&query_config(deps, env)?),
        BaseMinterQueryMsg::UserMintingDetails { address } => {
            to_json_binary(&query_user_minting_details(deps, env, address)?)
        }
        BaseMinterQueryMsg::TotalMintedCount {} => {
            to_json_binary(&query_total_tokens_minted(deps, env)?)
        }
        BaseMinterQueryMsg::IsPaused {} => to_json_binary(&query_is_paused(deps, env)?),
        BaseMinterQueryMsg::Pausers {} => to_json_binary(&query_pausers(deps, env)?),
        BaseMinterQueryMsg::AuthDetails {} => to_json_binary(&query_auth_details(deps, env)?),
        BaseMinterQueryMsg::Extension(ext) => match ext {
            OEMQueryExtension::TokensRemaining {} => {
                to_json_binary(&query_tokens_remaining(deps, env)?)
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

fn query_user_minting_details(
    deps: Deps,
    _env: Env,
    address: String,
) -> Result<UserDetails, ContractError> {
    let address = deps.api.addr_validate(&address)?;
    let user_details = USER_MINTING_DETAILS
        .load(deps.storage, address)
        .unwrap_or_default();
    Ok(user_details)
}

fn query_total_tokens_minted(deps: Deps, _env: Env) -> Result<u32, ContractError> {
    let total_tokens = MINTED_COUNT.load(deps.storage).unwrap_or(0);
    Ok(total_tokens)
}

fn query_tokens_remaining(deps: Deps, _env: Env) -> Result<u32, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if let Some(num_tokens) = config.num_tokens {
        let total_tokens = MINTED_COUNT.load(deps.storage).unwrap_or(0);
        Ok(num_tokens - total_tokens)
    } else {
        Err(ContractError::TokenLimitNotSet {})
    }
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
fn query_auth_details(deps: Deps, _env: Env) -> Result<AuthDetails, ContractError> {
    let auth_details = AUTH_DETAILS.load(deps.storage)?;
    Ok(auth_details)
}
fn query_mint_history(
    deps: Deps,
    _env: Env,
    address: String,
) -> Result<MintHistoryResponse, ContractError> {
    let address = deps.api.addr_validate(&address)?;
    let user_details = USER_MINTING_DETAILS
        .load(deps.storage, address)
        .unwrap_or_default();
    let config = CONFIG.load(deps.storage)?;
    let public_mint_limit = config.per_address_limit.unwrap_or(0);
    let total_minted_count = user_details.total_minted_count;
    let public_minted_count = user_details.public_mint_count;
    Ok(MintHistoryResponse {
        public_minted_count,
        public_mint_limit,
        total_minted_count,
    })
}
