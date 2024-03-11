#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Addr, Binary, Coin, CosmosMsg, Decimal, Deps, DepsMut, Env, MessageInfo,
    Response, StdResult, WasmMsg,
};
use cw_utils::{may_pay, maybe_addr, must_pay, nonpayable};
use minter_types::msg::QueryMsg as BaseMinterQueryMsg;
use minter_types::types::{
    AuthDetails, CollectionDetails, Config, Token, TokenDetails, UserDetails,
};
use minter_types::utils::{
    check_collection_creation_fee, generate_create_denom_msg, generate_mint_message,
    generate_update_denom_msg, update_collection_details,
};
use pauser::{PauseState, PAUSED_KEY, PAUSERS_KEY};
use std::str::FromStr;

use crate::drop::{
    get_drop_by_id, return_latest_drop_id, return_latest_drop_id_in_use, Drop, DropParams,
    ACTIVE_DROP_ID, DROPS, DROP_IDS_IN_USE, DROP_IDS_REMOVED,
};
use crate::error::ContractError;
use crate::msg::{ExecuteMsg, QueryMsgExtension};
use crate::state::{
    UserMintingDetails, AUTH_DETAILS, COLLECTION, LAST_MINTED_TOKEN_ID, USER_MINTING_DETAILS_KEY,
};

use cw2::set_contract_version;
use omniflix_open_edition_minter_factory::msg::{
    MultiMinterCreateMsg, ParamsResponse, QueryMsg as OpenEditionMinterFactoryQueryMsg,
};
use omniflix_round_whitelist::msg::ExecuteMsg as RoundWhitelistExecuteMsg;
use omniflix_std::types::omniflix::onft::v1beta1::{MsgPurgeDenom, WeightedAddress};
use whitelist_types::{
    check_if_address_is_member, check_if_whitelist_is_active, check_whitelist_price,
};

// version info for migration info
const CONTRACT_NAME: &str = "omniflix-multi-mint-open-edition-minter";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: MultiMinterCreateMsg,
) -> Result<Response, ContractError> {
    // Query denom creation fee
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    // Query factory params of instantiator
    // If the instantiator is not our factory then we wont be able to parse the response
    let _factory_params: ParamsResponse = deps.querier.query_wasm_smart(
        info.sender.clone().into_string(),
        &OpenEditionMinterFactoryQueryMsg::Params {},
    )?;

    let collection_creation_fee: Coin = check_collection_creation_fee(deps.as_ref().querier)?;

    let amount = must_pay(&info.clone(), &collection_creation_fee.denom)?;
    // Exact amount must be paid
    if amount != collection_creation_fee.amount {
        return Err(ContractError::InvalidCreationFee {
            expected: [Coin {
                denom: collection_creation_fee.denom,
                amount: collection_creation_fee.amount,
            }]
            .to_vec(),
            sent: info.funds.clone(),
        });
    };

    let admin = deps.api.addr_validate(&msg.init.admin)?;

    let payment_collector =
        maybe_addr(deps.api, msg.init.payment_collector.clone())?.unwrap_or(admin.clone());

    // Set the pause state
    let pause_state = PauseState::new(PAUSED_KEY, PAUSERS_KEY)?;
    pause_state.set_pausers(deps.storage, info.sender.clone(), vec![admin.clone()])?;

    let collection_details = msg.collection_details.clone();
    let auth_details = AuthDetails {
        admin: admin.clone(),
        payment_collector: payment_collector.clone(),
    };

    COLLECTION.save(deps.storage, &collection_details)?;
    // If drop id is zero then it means no drop is available
    ACTIVE_DROP_ID.save(deps.storage, &0)?;
    LAST_MINTED_TOKEN_ID.save(deps.storage, &0)?;
    AUTH_DETAILS.save(deps.storage, &auth_details)?;
    let nft_creation_fee = Coin {
        denom: collection_creation_fee.denom,
        amount: collection_creation_fee.amount,
    };
    let nft_creation_msg: CosmosMsg = generate_create_denom_msg(
        &collection_details,
        env.contract.address,
        nft_creation_fee,
        payment_collector,
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
        ExecuteMsg::SetAdmin { admin } => execute_set_admin(deps, env, info, admin),
        ExecuteMsg::SetPaymentCollector { payment_collector } => {
            execute_set_payment_collector(deps, env, info, payment_collector)
        }
        ExecuteMsg::Pause {} => execute_pause(deps, env, info),
        ExecuteMsg::Unpause {} => execute_unpause(deps, env, info),
        ExecuteMsg::SetPausers { pausers } => execute_set_pausers(deps, env, info, pausers),
        ExecuteMsg::NewDrop {
            config,
            token_details,
        } => execute_new_drop(deps, env, info, config, token_details),

        ExecuteMsg::UpdateRoyaltyReceivers { receivers } => {
            execute_update_royalty_receivers(deps, env, info, receivers)
        }
        ExecuteMsg::UpdateDenom {
            name,
            description,
            preview_uri,
        } => execute_update_denom(deps, env, info, name, description, preview_uri),
        ExecuteMsg::PurgeDenom {} => execute_purge_denom(deps, env, info),
        ExecuteMsg::RemoveDrop {} => execute_remove_drop(deps, info),
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
    let (drop_id, mut drop) = get_drop_by_id(drop_id, deps.storage)?;

    let collection_details = COLLECTION.load(deps.storage)?;
    let config = drop.clone().drop_params.config;
    let token_details = drop.clone().drop_params.token_details;
    let drop_minted_count = drop.clone().minted_count;
    let auth_details = AUTH_DETAILS.load(deps.storage)?;
    // Check if any token limit set and if it is reached
    if let Some(num_tokens) = config.num_tokens {
        if drop_minted_count >= num_tokens {
            return Err(ContractError::NoTokensLeftToMint {});
        }
    }

    // Check if end time is determined and if it is passed
    if let Some(end_time) = config.end_time {
        if env.block.time > end_time {
            return Err(ContractError::PublicMintingEnded {});
        }
    };
    let user_minting_details = UserMintingDetails::new(USER_MINTING_DETAILS_KEY);

    let mut user_details = user_minting_details
        .load(deps.storage, drop_id, info.sender.clone())
        .unwrap_or_default();

    // Load and increment the minted count
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
        // Check if per address limit is set and if it is reached
        if let Some(per_address_limit) = config.per_address_limit {
            if user_details.public_mint_count > per_address_limit {
                return Err(ContractError::AddressReachedMintLimit {});
            }
        }
    }
    // Increment total minted count
    user_details.total_minted_count += 1;

    user_details.minted_tokens.push(Token {
        token_id: token_id.to_string(),
    });
    // Save the user details
    user_minting_details.save(deps.storage, drop_id, info.sender.clone(), &user_details);

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
    let payment_collector = auth_details.payment_collector;

    // Increment the drop minted count and extract the drop token id
    drop.minted_count += 1;
    DROPS.save(deps.storage, drop_id, &drop)?;
    let drop_token_id = drop.minted_count;

    let mint_msg: CosmosMsg = generate_mint_message(
        &collection_details,
        &token_details,
        token_id.to_string(),
        env.contract.address,
        info.sender,
        Some((drop_token_id).to_string()),
        true,
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
        .add_attribute("drop_token_id", drop_token_id.to_string())
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

    // Find the drop
    let (drop_id, mut drop) = get_drop_by_id(drop_id, deps.storage)?;

    let collection_details = COLLECTION.load(deps.storage)?;
    let token_details = drop.clone().drop_params.token_details;
    let config = drop.clone().drop_params.config;
    let auth_details = AUTH_DETAILS.load(deps.storage)?;
    // Check if admin
    if info.sender != auth_details.admin {
        return Err(ContractError::Unauthorized {});
    }
    // Check if token limit is set and if it is reached
    if let Some(num_tokens) = config.num_tokens {
        if drop.minted_count >= num_tokens {
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

    let user_minting_details = UserMintingDetails::new(USER_MINTING_DETAILS_KEY);

    let mut user_details = user_minting_details
        .load(deps.storage, drop_id, recipient.clone())
        .unwrap_or_default();
    // We are only updating these params but not checking the mint limit
    user_details.total_minted_count += 1;
    user_details.minted_tokens.push(Token {
        token_id: token_id.to_string(),
    });

    // Save the user details
    user_minting_details.save(deps.storage, drop_id, recipient.clone(), &user_details);

    // Increment the drop minted count and extract the drop token id
    drop.minted_count += 1;
    DROPS.save(deps.storage, drop_id, &drop)?;
    let drop_token_id = drop.minted_count;

    let mint_msg: CosmosMsg = generate_mint_message(
        &collection_details,
        &token_details,
        token_id.to_string(),
        env.contract.address,
        recipient.clone(),
        Some((drop_token_id).to_string()),
        true,
    )
    .into();

    let res = Response::new()
        .add_message(mint_msg)
        .add_attribute("action", "mint")
        .add_attribute("token_id", token_id.to_string())
        .add_attribute("denom_id", collection_details.id)
        .add_attribute("drop_token_id", drop_token_id.to_string())
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
    let auth_details = AUTH_DETAILS.load(deps.storage)?;
    // Check if sender is admin
    if info.sender != auth_details.admin {
        return Err(ContractError::Unauthorized {});
    }
    // Find the drop
    let (drop_id, mut drop) = get_drop_by_id(drop_id, deps.storage)?;
    // Extract the token details
    let mut drop_token_details = drop.clone().drop_params.token_details;
    // Set the new ratio without checking the ratio
    drop_token_details.royalty_ratio = Decimal::from_str(&ratio)?;
    // Check integrity of token details
    drop_token_details.check_integrity()?;

    // Save the new token details
    drop.drop_params.token_details.royalty_ratio = Decimal::from_str(&ratio)?;
    DROPS.save(deps.storage, drop_id, &drop)?;

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
    let auth_details = AUTH_DETAILS.load(deps.storage)?;

    // Check if sender is admin
    if info.sender != auth_details.admin {
        return Err(ContractError::Unauthorized {});
    }
    // Find the drop
    let (drop_id, mut drop) = get_drop_by_id(drop_id, deps.storage)?;
    drop.drop_params.config.mint_price = mint_price.clone();

    DROPS.save(deps.storage, drop_id, &drop)?;

    let res = Response::new()
        .add_attribute("action", "update_mint_price")
        .add_attribute("mint_price_denom", mint_price.denom.to_string())
        .add_attribute("mint_price_amount", mint_price.amount.to_string())
        .add_attribute("drop_id", drop_id.to_string());
    Ok(res)
}

pub fn execute_set_admin(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    admin: String,
) -> Result<Response, ContractError> {
    let mut auth_details = AUTH_DETAILS.load(deps.storage)?;
    // Check if sender is admin
    if info.sender != auth_details.admin {
        return Err(ContractError::Unauthorized {});
    }

    let validated_new_admin = deps.api.addr_validate(&admin)?;
    auth_details.admin = validated_new_admin.clone();
    AUTH_DETAILS.save(deps.storage, &auth_details)?;

    let res = Response::new()
        .add_attribute("action", "set_admin")
        .add_attribute("admin", admin.to_string());
    Ok(res)
}

pub fn execute_set_payment_collector(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    payment_collector: String,
) -> Result<Response, ContractError> {
    let mut auth_details = AUTH_DETAILS.load(deps.storage)?;
    // Check if sender is admin
    if info.sender != auth_details.admin {
        return Err(ContractError::Unauthorized {});
    }
    let validated_new_payment_collector = deps.api.addr_validate(&payment_collector)?;
    auth_details.payment_collector = validated_new_payment_collector.clone();
    AUTH_DETAILS.save(deps.storage, &auth_details)?;

    let res = Response::new()
        .add_attribute("action", "set_payment_collector")
        .add_attribute("payment_collector", payment_collector.to_string());
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
    let auth_details = AUTH_DETAILS.load(deps.storage)?;
    if info.sender != auth_details.admin {
        return Err(ContractError::Unauthorized {});
    }
    // Find the drop
    let (drop_id, mut drop) = get_drop_by_id(drop_id, deps.storage)?;

    let current_whitelist_address = drop.drop_params.config.whitelist_address.clone();

    // Check if current whitelist already active
    if let Some(current_whitelist_address) = current_whitelist_address {
        let is_active: bool =
            check_if_whitelist_is_active(&current_whitelist_address, deps.as_ref())?;
        if is_active {
            return Err(ContractError::WhitelistAlreadyActive {});
        }
    }

    let validated_new_whitelist_address = deps.api.addr_validate(&address)?;

    let is_active: bool =
        check_if_whitelist_is_active(&validated_new_whitelist_address, deps.as_ref())?;
    if is_active {
        return Err(ContractError::WhitelistAlreadyActive {});
    }
    // Set the new whitelist address
    drop.drop_params.config.whitelist_address = Some(validated_new_whitelist_address.clone());
    DROPS.save(deps.storage, drop_id, &drop)?;

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
    let auth_details = AUTH_DETAILS.load(deps.storage)?;

    if info.sender != auth_details.admin {
        return Err(ContractError::Unauthorized {});
    }
    // Check integrity of token details
    token_details.check_integrity()?;
    // Check if token limit is 0
    if let Some(num_tokens) = config.num_tokens {
        if num_tokens == 0 {
            return Err(ContractError::InvalidNumTokens {});
        }
    }
    // Check if per address limit is 0
    if let Some(per_address_limit) = config.per_address_limit {
        if per_address_limit == 0 {
            return Err(ContractError::PerAddressLimitZero {});
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
    let new_drop_params = DropParams {
        config,
        token_details,
    };
    let new_drop = Drop {
        minted_count: 0,
        drop_params: new_drop_params.clone(),
    };

    let latest_drop_id = return_latest_drop_id(deps.storage)?;
    let new_drop_id = latest_drop_id + 1;
    DROPS.save(deps.storage, new_drop_id, &new_drop)?;

    let mut drop_ids_in_use = DROP_IDS_IN_USE.load(deps.storage).unwrap_or_default();
    drop_ids_in_use.push(new_drop_id);

    DROP_IDS_IN_USE.save(deps.storage, &drop_ids_in_use)?;
    ACTIVE_DROP_ID.save(deps.storage, &new_drop_id)?;

    let res = Response::new()
        .add_attribute("action", "new_drop")
        .add_attribute("new_drop_id", new_drop_id.to_string());

    Ok(res)
}

pub fn execute_remove_drop(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
    let auth_details = AUTH_DETAILS.load(deps.storage)?;
    // Check if sender is admin
    if info.sender != auth_details.admin {
        return Err(ContractError::Unauthorized {});
    }
    // The active drop will always be the biggest drop ID in use.
    let active_drop_id = ACTIVE_DROP_ID.load(deps.storage)?;
    // If active drop id is 0 then no drop is available
    if active_drop_id == 0 {
        return Err(ContractError::NoDropAvailable {});
    }
    let (drop_id, drop) = get_drop_by_id(Some(active_drop_id), deps.storage)?;

    if drop.minted_count > 0 {
        return Err(ContractError::DropCantBeRemoved {});
    }
    DROPS.remove(deps.storage, drop_id);
    let mut drop_ids_in_use = DROP_IDS_IN_USE.load(deps.storage)?;

    // Remove the drop id from the list
    drop_ids_in_use.retain(|&x| x != drop_id);
    DROP_IDS_IN_USE.save(deps.storage, &drop_ids_in_use)?;

    let mut drop_ids_removed = DROP_IDS_REMOVED.load(deps.storage).unwrap_or_default();

    // Add the drop id to the removed list
    drop_ids_removed.push(drop_id);
    DROP_IDS_REMOVED.save(deps.storage, &drop_ids_removed)?;

    let new_active_drop_id = return_latest_drop_id_in_use(deps.storage)?;
    ACTIVE_DROP_ID.save(deps.storage, &new_active_drop_id)?;

    Ok(Response::new()
        .add_attribute("action", "remove_drop")
        .add_attribute("new_active_drop_id", new_active_drop_id.to_string())
        .add_attribute("removed_drop_id", drop_id.to_string()))
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
    // Check if sender is admin
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
    name: Option<String>,
    description: Option<String>,
    preview_uri: Option<String>,
) -> Result<Response, ContractError> {
    let auth_details = AUTH_DETAILS.load(deps.storage)?;
    let admin = auth_details.admin;
    // Current drops admin can update the denom
    if info.sender != admin {
        return Err(ContractError::Unauthorized {});
    }
    let collection_details = COLLECTION.load(deps.storage)?;
    let new_collection_details =
        update_collection_details(&collection_details, name, description, preview_uri, None);

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
    let auth_details = AUTH_DETAILS.load(deps.storage)?;

    if auth_details.admin != info.sender {
        return Err(ContractError::Unauthorized {});
    }
    let collection_details = COLLECTION.load(deps.storage)?;

    let purge_msg: CosmosMsg = MsgPurgeDenom {
        id: collection_details.id,
        sender: env.contract.address.into_string(),
    }
    .into();

    Ok(Response::new()
        .add_attribute("action", "purge_denom")
        .add_message(purge_msg))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(
    deps: Deps,
    env: Env,
    msg: BaseMinterQueryMsg<QueryMsgExtension>,
) -> StdResult<Binary> {
    match msg {
        BaseMinterQueryMsg::Collection {} => to_json_binary(&query_collection(deps, env)?),
        BaseMinterQueryMsg::TokenDetails {} => {
            to_json_binary(&query_token_details(deps, env, None)?)
        }
        BaseMinterQueryMsg::Config {} => to_json_binary(&query_config(deps, env, None)?),
        BaseMinterQueryMsg::UserMintingDetails { address } => {
            to_json_binary(&query_user_minting_details(deps, env, address, None)?)
        }
        BaseMinterQueryMsg::TotalMintedCount {} => {
            to_json_binary(&query_total_tokens_minted(deps, env)?)
        }
        BaseMinterQueryMsg::AuthDetails {} => to_json_binary(&query_auth_details(deps, env)?),
        BaseMinterQueryMsg::IsPaused {} => to_json_binary(&query_is_paused(deps, env)?),
        BaseMinterQueryMsg::Pausers {} => to_json_binary(&query_pausers(deps, env)?),
        BaseMinterQueryMsg::Extension(ext) => match ext {
            QueryMsgExtension::ActiveDropId {} => to_json_binary(&query_active_drop_id(deps, env)?),
            QueryMsgExtension::AllDrops {} => to_json_binary(&query_all_drops(deps, env)?),
            QueryMsgExtension::Config { drop_id } => {
                to_json_binary(&query_config(deps, env, drop_id)?)
            }
            QueryMsgExtension::UserMintingDetails { address, drop_id } => {
                to_json_binary(&query_user_minting_details(deps, env, address, drop_id)?)
            }
            QueryMsgExtension::TokensRemainingInDrop { drop_id } => {
                to_json_binary(&query_tokens_remaining_in_drop(deps, env, drop_id)?)
            }
            QueryMsgExtension::TokenDetails { drop_id } => {
                to_json_binary(&query_token_details(deps, env, drop_id)?)
            }
            QueryMsgExtension::TokensMintedInDrop { drop_id } => {
                to_json_binary(&query_tokens_minted_in_drop(deps, env, drop_id)?)
            }
        },
    }
}

fn query_collection(deps: Deps, _env: Env) -> Result<CollectionDetails, ContractError> {
    let collection = COLLECTION.load(deps.storage)?;
    Ok(collection)
}
fn query_token_details(
    deps: Deps,
    _env: Env,
    drop_id: Option<u32>,
) -> Result<TokenDetails, ContractError> {
    // Find the drop
    let (_, drop) = get_drop_by_id(drop_id, deps.storage)?;
    let token_details = drop.drop_params.token_details;
    Ok(token_details)
}

fn query_config(deps: Deps, _env: Env, drop_id: Option<u32>) -> Result<Config, ContractError> {
    // Find the drop
    let (_, drop) = get_drop_by_id(drop_id, deps.storage)?;
    let config = drop.drop_params.config;
    Ok(config)
}

fn query_user_minting_details(
    deps: Deps,
    _env: Env,
    address: String,
    drop_id: Option<u32>,
) -> Result<UserDetails, ContractError> {
    let address = deps.api.addr_validate(&address)?;
    let drop_id = drop_id.unwrap_or(ACTIVE_DROP_ID.load(deps.storage)?);
    let user_minting_details = UserMintingDetails::new(USER_MINTING_DETAILS_KEY);
    let user_details = user_minting_details
        .load(deps.storage, drop_id, address)
        .unwrap_or_default();
    Ok(user_details)
}

fn query_total_tokens_minted(deps: Deps, _env: Env) -> Result<u32, ContractError> {
    let total_minted_count = LAST_MINTED_TOKEN_ID.load(deps.storage)?;
    Ok(total_minted_count)
}

fn query_tokens_remaining_in_drop(
    deps: Deps,
    _env: Env,
    drop_id: Option<u32>,
) -> Result<u32, ContractError> {
    let (_, drop) = get_drop_by_id(drop_id, deps.storage)?;
    let config = drop.drop_params.config;
    let drop_minted_count = drop.minted_count;

    if let Some(num_tokens) = config.num_tokens {
        let tokens_remaining = num_tokens - drop_minted_count;
        Ok(tokens_remaining)
    } else {
        Err(ContractError::TokenLimitNotSet {})
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

fn query_active_drop_id(deps: Deps, _env: Env) -> Result<u32, ContractError> {
    let active_drop_id = ACTIVE_DROP_ID.load(deps.storage).unwrap_or(0);
    Ok(active_drop_id)
}

fn query_all_drops(deps: Deps, _env: Env) -> Result<Vec<(u32, Drop)>, ContractError> {
    let drop_ids_in_use = DROP_IDS_IN_USE.load(deps.storage).unwrap_or_default();

    let mut drops: Vec<(u32, Drop)> = vec![];

    if drop_ids_in_use.is_empty() {
        return Ok(drops);
    }

    for drop_id in drop_ids_in_use {
        let (_, drop) = get_drop_by_id(Some(drop_id), deps.storage)?;
        drops.push((drop_id, drop));
    }
    Ok(drops)
}

fn query_auth_details(deps: Deps, _env: Env) -> Result<AuthDetails, ContractError> {
    let auth_details = AUTH_DETAILS.load(deps.storage)?;
    Ok(auth_details)
}

fn query_tokens_minted_in_drop(
    deps: Deps,
    _env: Env,
    drop_id: Option<u32>,
) -> Result<u32, ContractError> {
    let (_, drop) = get_drop_by_id(drop_id, deps.storage)?;
    let drop_minted_count = drop.minted_count;
    Ok(drop_minted_count)
}
