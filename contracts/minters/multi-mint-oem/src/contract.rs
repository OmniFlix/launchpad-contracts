#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Addr, Binary, Coin, CosmosMsg, Decimal, Deps, DepsMut, Env, MessageInfo,
    Response, StdResult, Uint128, WasmMsg,
};
use cw_utils::{may_pay, maybe_addr, must_pay, nonpayable};
use minter_types::{
    generate_create_denom_msg, generate_mint_message, AuthDetails, CollectionDetails, Config,
    QueryMsg as MinterQueryMsg, Token, TokenDetails, UserDetails,
};
use omniflix_std::types::cosmos::auth;
use pauser::{PauseState, PAUSED_KEY, PAUSERS_KEY};
use std::str::FromStr;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, QueryMsgExtension};
use crate::state::{
    DropParams, UserMintedTokens, AUTH_DETAILS, CURRENT_DROP_ID, DROPS, DROP_MINTED_COUNT,
    LAST_MINTED_TOKEN_ID, USER_MINTED_TOKENS_KEY,
};

use cw2::set_contract_version;
use omniflix_open_edition_minter_factory::msg::{
    OpenEditionMinterCreateMsg, ParamsResponse, QueryMsg as OpenEditionMinterFactoryQueryMsg,
};
use omniflix_round_whitelist::msg::ExecuteMsg as RoundWhitelistExecuteMsg;
use omniflix_std::types::omniflix::onft::v1beta1::{
    MsgPurgeDenom, MsgUpdateDenom, OnftQuerier, WeightedAddress,
};
use whitelist_types::{
    check_if_address_is_member, check_if_whitelist_is_active, check_whitelist_price,
};

// version info for migration info
const CONTRACT_NAME: &str = "omniflix-multi-mint-open-edition-minter";
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
    if let Some(num_tokens) = msg.init.num_tokens {
        if num_tokens == 0 {
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
    let royalty_ratio = msg.token_details.royalty_ratio.clone();

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
        maybe_addr(deps.api, msg.init.payment_collector.clone())?.unwrap_or(admin.clone());

    let config = Config {
        per_address_limit: msg.init.per_address_limit,
        start_time: msg.init.start_time,
        mint_price: msg.init.mint_price,
        whitelist_address: maybe_addr(deps.api, msg.init.whitelist_address.clone())?,
        end_time: msg.init.end_time,
        num_tokens: msg.init.num_tokens,
    };
    // Set the pause state
    let pause_state = PauseState::new(PAUSED_KEY, PAUSERS_KEY)?;
    pause_state.set_pausers(deps.storage, info.sender.clone(), vec![admin.clone()])?;

    let collection_details = msg.collection_details.clone();
    let token_details = msg.token_details.clone();
    let auth_details = AuthDetails {
        admin: admin.clone(),
        payment_collector: payment_collector.clone(),
    };

    // Save the collection as drop 1
    let drop_params = DropParams {
        config: config.clone(),
        collection_details: collection_details.clone(),
        token_details: token_details,
    };

    DROPS.save(deps.storage, 1, &drop_params)?;
    CURRENT_DROP_ID.save(deps.storage, &1)?;
    DROP_MINTED_COUNT.save(deps.storage, 1, &0)?;
    LAST_MINTED_TOKEN_ID.save(deps.storage, &0)?;
    AUTH_DETAILS.save(deps.storage, &auth_details)?;
    let nft_creation_fee = Coin {
        denom: creation_fee_denom,
        amount: creation_fee_amount,
    };
    let nft_creation_msg: CosmosMsg = generate_create_denom_msg(
        &collection_details,
        env.contract.address,
        nft_creation_fee,
        admin,
    )?
    .into();

    let res = Response::new()
        .add_message(nft_creation_msg)
        .add_attribute("action", "instantiate")
        .add_attribute("contract_version", CONTRACT_VERSION)
        .add_attribute("contract_name", CONTRACT_NAME)
        .add_attribute("drop_id", "1");

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
        ExecuteMsg::Mint { drop_id } => execute_mint(deps, env, info, drop_id),
        ExecuteMsg::MintAdmin { recipient, drop_id } => {
            execute_mint_admin(deps, env, info, recipient, drop_id)
        }
        ExecuteMsg::UpdateRoyaltyRatio { ratio, drop_id } => {
            execute_update_royalty_ratio(deps, env, info, ratio, drop_id)
        }
        ExecuteMsg::UpdateMintPrice {
            mint_price,
            drop_id,
        } => execute_update_mint_price(deps, env, info, mint_price, drop_id),
        ExecuteMsg::UpdateWhitelistAddress { address, drop_id } => {
            execute_update_whitelist_address(deps, env, info, address, drop_id)
        }
        ExecuteMsg::Pause {} => execute_pause(deps, env, info),
        ExecuteMsg::Unpause {} => execute_unpause(deps, env, info),
        ExecuteMsg::SetPausers { pausers } => execute_set_pausers(deps, env, info, pausers),
        ExecuteMsg::NewDrop {
            new_config,
            new_token_details,
        } => execute_new_drop(deps, env, info, new_config, new_token_details),

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

pub fn execute_mint(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    drop_id: Option<u32>,
) -> Result<Response, ContractError> {
    let pause_state = PauseState::new(PAUSED_KEY, PAUSERS_KEY)?;
    pause_state.error_if_paused(deps.storage)?;

    // Find the drop
    let drop_id = drop_id.unwrap_or(CURRENT_DROP_ID.load(deps.storage)?);
    let drop_params = DROPS.load(deps.storage, drop_id)?;

    let config = drop_params.config;
    let collection_details = drop_params.collection_details;
    let token_details = drop_params.token_details;
    let auth_details = AUTH_DETAILS.load(deps.storage)?;
    // Check if any token limit set and if it is reached
    if let Some(num_tokens) = config.num_tokens {
        if DROP_MINTED_COUNT.load(deps.storage, drop_id).unwrap_or(0) >= num_tokens {
            return Err(ContractError::NoTokensLeftToMint {});
        }
    }

    // Check if end time is determined and if it is passed
    if let Some(end_time) = config.end_time {
        if env.block.time > end_time {
            return Err(ContractError::PublicMintingEnded {});
        }
    };
    let user_minted_tokens = UserMintedTokens::new(USER_MINTED_TOKENS_KEY);

    let mut user_details = user_minted_tokens
        .load(deps.storage, drop_id, info.sender.clone())
        .unwrap_or_default();
    // Load and increment the minted count
    // This minted count is seperate from the drops
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
        // Check if address has reached the public mint limit
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
    user_minted_tokens.save(deps.storage, drop_id, info.sender.clone(), &user_details);

    // Check the payment
    // Can be set to zero so use may_pay
    let amount = may_pay(&info, &mint_price.denom)?;
    // Exact amount must be paid
    if amount != mint_price.amount {
        return Err(ContractError::IncorrectPaymentAmount {
            expected: mint_price.amount,
            sent: amount,
        });
    }
    // Get the payment collector address
    let payment_collector = auth_details.payment_collector;

    let mut drop_minted_count = DROP_MINTED_COUNT.load(deps.storage, drop_id)?;
    drop_minted_count += 1;
    DROP_MINTED_COUNT.save(deps.storage, drop_id, &drop_minted_count)?;

    let mint_msg: CosmosMsg = generate_mint_message(
        &collection_details,
        &token_details,
        token_id.to_string(),
        env.contract.address,
        info.sender,
        Some((drop_minted_count).to_string()),
        is_public,
    )
    .into();

    if !mint_price.amount.is_zero() {
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
        .add_attribute("drop_token_id", drop_minted_count.to_string())
        .add_attribute("collection_id", collection_details.id)
        .add_attribute("drop_id", drop_id.to_string());

    Ok(res)
}

pub fn execute_mint_admin(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    recipient: String,
    drop_id: Option<u32>,
) -> Result<Response, ContractError> {
    nonpayable(&info)?;

    let drop_id = drop_id.unwrap_or(CURRENT_DROP_ID.load(deps.storage)?);
    let drop_params = DROPS.load(deps.storage, drop_id)?;

    let config = drop_params.config;
    let collection_details = drop_params.collection_details;
    let token_details = drop_params.token_details;
    let auth_details = AUTH_DETAILS.load(deps.storage)?;
    // Check if admin
    if info.sender != auth_details.admin {
        return Err(ContractError::Unauthorized {});
    }
    // Check if any token left for current drop
    if let Some(num_tokens) = config.num_tokens {
        if DROP_MINTED_COUNT.load(deps.storage, drop_id).unwrap_or(0) >= num_tokens {
            return Err(ContractError::NoTokensLeftToMint {});
        }
    }
    // Check if end time is determined and if it is passed
    if let Some(end_time) = config.end_time {
        if env.block.time > end_time {
            return Err(ContractError::PublicMintingEnded {});
        }
    };

    let pause_state = PauseState::new(PAUSED_KEY, PAUSERS_KEY)?;
    pause_state.error_if_paused(deps.storage)?;

    let recipient = deps.api.addr_validate(&recipient)?;
    let last_token_id = LAST_MINTED_TOKEN_ID.load(deps.storage)?;
    let token_id = last_token_id + 1;

    let user_minted_tokens = UserMintedTokens::new(USER_MINTED_TOKENS_KEY);

    let mut user_details = user_minted_tokens
        .load(deps.storage, drop_id, recipient.clone())
        .unwrap_or_default();
    // We are only updating these params but not checking the mint limit
    user_details.total_minted_count += 1;
    user_details.minted_tokens.push(Token {
        token_id: token_id.to_string(),
    });

    // Save the user details
    user_minted_tokens.save(deps.storage, drop_id, recipient.clone(), &user_details);

    // Load current drops minted count and increment it
    let mut drop_minted_count = DROP_MINTED_COUNT.load(deps.storage, drop_id)?;
    drop_minted_count += 1;
    DROP_MINTED_COUNT.save(deps.storage, drop_id, &drop_minted_count)?;

    let mint_msg: CosmosMsg = generate_mint_message(
        &collection_details,
        &token_details,
        token_id.to_string(),
        env.contract.address,
        recipient.clone(),
        Some((drop_minted_count).to_string()),
        true,
    )
    .into();

    let res = Response::new()
        .add_message(mint_msg)
        .add_attribute("action", "mint")
        .add_attribute("token_id", token_id.to_string())
        .add_attribute("denom_id", collection_details.id)
        .add_attribute("drop_token_id", drop_minted_count.to_string())
        .add_attribute("drop_id", drop_id.to_string());
    Ok(res)
}

pub fn execute_update_royalty_ratio(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    ratio: String,
    drop_id: Option<u32>,
) -> Result<Response, ContractError> {
    let drop_id = drop_id.unwrap_or(CURRENT_DROP_ID.load(deps.storage)?);
    let mut drop_params = DROPS.load(deps.storage, drop_id)?;
    let auth_details = AUTH_DETAILS.load(deps.storage)?;
    // Check if sender is admin
    if info.sender != auth_details.admin {
        return Err(ContractError::Unauthorized {});
    }

    // Check if ratio is decimal number
    let ratio = Decimal::from_str(&ratio)?;

    if ratio < Decimal::zero() || ratio > Decimal::one() {
        return Err(ContractError::InvalidRoyaltyRatio {});
    }

    drop_params.token_details.royalty_ratio = ratio;

    DROPS.save(deps.storage, drop_id, &drop_params)?;

    let res = Response::new()
        .add_attribute("action", "update_royalty_ratio")
        .add_attribute("ratio", ratio.to_string())
        .add_attribute("drop_id", drop_id.to_string());
    Ok(res)
}

pub fn execute_update_mint_price(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    mint_price: Coin,
    drop_id: Option<u32>,
) -> Result<Response, ContractError> {
    let drop_id = drop_id.unwrap_or(CURRENT_DROP_ID.load(deps.storage)?);
    let mut drop_params = DROPS.load(deps.storage, drop_id)?;
    let auth_details = AUTH_DETAILS.load(deps.storage)?;

    // Check if sender is admin
    if info.sender != auth_details.admin {
        return Err(ContractError::Unauthorized {});
    }
    drop_params.config.mint_price = mint_price.clone();

    DROPS.save(deps.storage, drop_id, &drop_params)?;

    let res = Response::new()
        .add_attribute("action", "update_mint_price")
        .add_attribute("mint_price_denom", mint_price.denom.to_string())
        .add_attribute("mint_price_amount", mint_price.amount.to_string())
        .add_attribute("drop_id", drop_id.to_string());
    Ok(res)
}

pub fn execute_update_whitelist_address(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    address: String,
    drop_id: Option<u32>,
) -> Result<Response, ContractError> {
    // Check if sender is admin
    let drop_id = drop_id.unwrap_or(CURRENT_DROP_ID.load(deps.storage)?);
    let mut drop_params = DROPS.load(deps.storage, drop_id)?;
    let auth_details = AUTH_DETAILS.load(deps.storage)?;
    // Check if sender is admin
    if info.sender != auth_details.admin {
        return Err(ContractError::Unauthorized {});
    }

    let whitelist_address = drop_params.config.whitelist_address.clone();

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
    drop_params.config.whitelist_address = Some(address.clone());
    DROPS.save(deps.storage, drop_id, &drop_params)?;

    let res = Response::new()
        .add_attribute("action", "update_whitelist_address")
        .add_attribute("address", address.to_string())
        .add_attribute("drop_id", drop_id.to_string());
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

pub fn execute_new_drop(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    config: Config,
    token_details: TokenDetails,
) -> Result<Response, ContractError> {
    // Check if sender is admin
    let current_drop_id = CURRENT_DROP_ID.load(deps.storage)?;
    let current_drop_params = DROPS.load(deps.storage, current_drop_id)?;
    let auth_details = AUTH_DETAILS.load(deps.storage)?;

    if info.sender != auth_details.admin {
        return Err(ContractError::Unauthorized {});
    }
    // Check if token limit is 0
    if let Some(num_tokens) = config.num_tokens {
        if num_tokens == 0 {
            return Err(ContractError::InvalidNumTokens {});
        }
    }
    // Check start time
    if config.start_time < env.block.time {
        return Err(ContractError::InvalidStartTime {});
    }
    // Check end time
    if let Some(end_time) = config.end_time {
        if end_time < config.start_time {
            return Err(ContractError::InvalidEndTime {});
        }
    }
    // Check if any whitelist is present
    if let Some(whitelist_address) = config.whitelist_address.clone() {
        let is_active: bool = check_if_whitelist_is_active(
            &deps.api.addr_validate(&whitelist_address.into_string())?,
            deps.as_ref(),
        )?;
        if is_active {
            return Err(ContractError::WhitelistAlreadyActive {});
        }
    }
    // Check royalty ratio we expect decimal number
    let royalty_ratio = token_details.royalty_ratio.clone();

    let new_drop_params = DropParams {
        config: config.clone(),
        collection_details: current_drop_params.collection_details,
        token_details: token_details,
    };
    let new_drop_id = current_drop_id + 1;
    DROPS.save(deps.storage, new_drop_id, &new_drop_params)?;
    CURRENT_DROP_ID.save(deps.storage, &new_drop_id)?;
    DROP_MINTED_COUNT.save(deps.storage, new_drop_id, &0)?;

    let res = Response::new()
        .add_attribute("action", "new_drop")
        .add_attribute("new_drop_id", new_drop_id.to_string());

    Ok(res)
}
pub fn execute_update_royalty_receivers(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    receivers: Vec<WeightedAddress>,
) -> Result<Response, ContractError> {
    // Check if sender is admin
    let current_drop_id = CURRENT_DROP_ID.load(deps.storage)?;
    let drop_params = DROPS.load(deps.storage, current_drop_id)?;
    let auth_details = AUTH_DETAILS.load(deps.storage)?;
    // Check if sender is admin
    if info.sender != auth_details.admin {
        return Err(ContractError::Unauthorized {});
    }

    let update_msg: CosmosMsg = MsgUpdateDenom {
        sender: env.contract.address.into_string(),
        id: drop_params.collection_details.id,
        description: "[do-not-modify]".to_string(),
        name: "[do-not-modify]".to_string(),
        preview_uri: "[do-not-modify]".to_string(),
        royalty_receivers: receivers,
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
    let current_drop_id = CURRENT_DROP_ID.load(deps.storage)?;
    let current_drop_params = DROPS.load(deps.storage, current_drop_id)?;
    let auth_details = AUTH_DETAILS.load(deps.storage)?;
    let admin = auth_details.admin;
    // Current drops admin can update the denom
    if info.sender != admin {
        return Err(ContractError::Unauthorized {});
    }
    // We would have to update name and description in the collection of all drops
    for edition_number in 1..=current_drop_id {
        let mut edition_params = DROPS.load(deps.storage, edition_number)?;
        edition_params.collection_details.collection_name = name
            .clone()
            .unwrap_or(edition_params.collection_details.collection_name);
        edition_params.collection_details.description = description.clone();
        edition_params.collection_details.preview_uri = preview_uri.clone();

        DROPS.save(deps.storage, edition_number, &edition_params)?;
    }

    let update_msg: CosmosMsg = MsgUpdateDenom {
        sender: env.contract.address.into_string(),
        id: current_drop_params.collection_details.id,
        description: description.unwrap_or("[do-not-modify]".to_string()),
        name: name.unwrap_or("[do-not-modify]".to_string()),
        preview_uri: preview_uri.unwrap_or("[do-not-modify]".to_string()),
        royalty_receivers: vec![],
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
    let current_drop_id = CURRENT_DROP_ID.load(deps.storage)?;
    let current_drop_params = DROPS.load(deps.storage, current_drop_id)?;
    let auth_details = AUTH_DETAILS.load(deps.storage)?;

    if auth_details.admin != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    let purge_msg: CosmosMsg = MsgPurgeDenom {
        id: current_drop_params.collection_details.id,
        sender: env.contract.address.into_string(),
    }
    .into();

    Ok(Response::new()
        .add_attribute("action", "purge_denom")
        .add_message(purge_msg))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: MinterQueryMsg<QueryMsgExtension>) -> StdResult<Binary> {
    match msg {
        MinterQueryMsg::Collection {} => to_json_binary(&query_collection(deps, env, None)?),
        MinterQueryMsg::TokenDetails {} => to_json_binary(&query_token_details(deps, env, None)?),
        MinterQueryMsg::Config {} => to_json_binary(&query_config(deps, env, None)?),
        MinterQueryMsg::MintedTokens { address } => {
            to_json_binary(&query_minted_tokens(deps, env, address, None)?)
        }
        MinterQueryMsg::TotalMintedCount {} => {
            to_json_binary(&query_total_tokens_minted(deps, env)?)
        }
        MinterQueryMsg::IsPaused {} => to_json_binary(&query_is_paused(deps, env)?),
        MinterQueryMsg::Pausers {} => to_json_binary(&query_pausers(deps, env)?),
        MinterQueryMsg::Extension(ext) => match ext {
            QueryMsgExtension::CurrentDropNumber {} => {
                to_json_binary(&query_current_drop_number(deps, env)?)
            }
            QueryMsgExtension::AllDrops {} => to_json_binary(&query_all_drops(deps, env)?),
            QueryMsgExtension::Collection { drop_id } => {
                to_json_binary(&query_collection(deps, env, drop_id)?)
            }
            QueryMsgExtension::Config { drop_id } => {
                to_json_binary(&query_config(deps, env, drop_id)?)
            }
            QueryMsgExtension::MintedTokens { address, drop_id } => {
                to_json_binary(&query_minted_tokens(deps, env, address, drop_id)?)
            }
            QueryMsgExtension::TokensRemaining { drop_id } => {
                to_json_binary(&query_tokens_remaining(deps, env, drop_id)?)
            }
            QueryMsgExtension::TokenDetails { drop_id } => {
                to_json_binary(&query_token_details(deps, env, drop_id)?)
            }
        },
    }
}

fn query_collection(
    deps: Deps,
    _env: Env,
    drop_id: Option<u32>,
) -> Result<CollectionDetails, ContractError> {
    let drop_id = drop_id.unwrap_or(CURRENT_DROP_ID.load(deps.storage)?);
    let collection = DROPS.load(deps.storage, drop_id)?.collection_details;
    Ok(collection)
}
fn query_token_details(
    deps: Deps,
    _env: Env,
    drop_id: Option<u32>,
) -> Result<TokenDetails, ContractError> {
    let drop_id = drop_id.unwrap_or(CURRENT_DROP_ID.load(deps.storage)?);
    let token_details = DROPS.load(deps.storage, drop_id)?.token_details;
    Ok(token_details)
}

fn query_config(deps: Deps, _env: Env, drop_id: Option<u32>) -> Result<Config, ContractError> {
    let drop_id = drop_id.unwrap_or(CURRENT_DROP_ID.load(deps.storage)?);
    let drop_params = DROPS.load(deps.storage, drop_id)?;
    let config = drop_params.config;
    Ok(config)
}

fn query_minted_tokens(
    deps: Deps,
    _env: Env,
    address: String,
    drop_id: Option<u32>,
) -> Result<UserDetails, ContractError> {
    let address = deps.api.addr_validate(&address)?;
    let drop_id = drop_id.unwrap_or(CURRENT_DROP_ID.load(deps.storage)?);
    let minted_tokens = UserMintedTokens::new(USER_MINTED_TOKENS_KEY);
    let user_details = minted_tokens.load(deps.storage, drop_id, address)?;
    Ok(user_details)
}

fn query_total_tokens_minted(deps: Deps, _env: Env) -> Result<u32, ContractError> {
    let total_minted_count = LAST_MINTED_TOKEN_ID.load(deps.storage)?;
    Ok(total_minted_count)
}

fn query_tokens_remaining(
    deps: Deps,
    _env: Env,
    drop_id: Option<u32>,
) -> Result<u32, ContractError> {
    let drop_id = drop_id.unwrap_or(CURRENT_DROP_ID.load(deps.storage)?);
    let drop_params = DROPS.load(deps.storage, drop_id)?;
    let config = drop_params.config;
    let drop_minted_count = DROP_MINTED_COUNT.load(deps.storage, drop_id).unwrap_or(0);
    if let Some(num_tokens) = config.num_tokens {
        let tokens_remaining = num_tokens - drop_minted_count;
        return Ok(tokens_remaining);
    } else {
        return Err(ContractError::TokenLimitNotSet {});
    }
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

fn query_current_drop_number(deps: Deps, _env: Env) -> Result<u32, ContractError> {
    let current_edition = CURRENT_DROP_ID.load(deps.storage).unwrap_or(1);
    Ok(current_edition)
}

fn query_all_drops(deps: Deps, _env: Env) -> Result<Vec<(u32, DropParams)>, ContractError> {
    let current_edition = CURRENT_DROP_ID.load(deps.storage).unwrap_or(1);
    let mut drops: Vec<(u32, DropParams)> = vec![];
    for edition_number in 1..=current_edition {
        let drop_params = DROPS.load(deps.storage, edition_number)?;
        drops.push((edition_number, drop_params));
    }
    Ok(drops)
}
