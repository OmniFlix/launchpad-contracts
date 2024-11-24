#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Addr, Binary, Coin, CosmosMsg, Decimal, Deps, DepsMut, Env, MessageInfo,
    Response, StdResult, WasmMsg,
};
use cw_utils::{may_pay, must_pay, nonpayable};
use minter_types::collection_details::{update_collection_details, CollectionDetails};
use minter_types::config::Config;
use minter_types::msg::{MintHistoryResponse, QueryMsg as BaseMinterQueryMsg};
use minter_types::token_details::{Token, TokenDetails};
use minter_types::types::{AuthDetails, UserDetails};
use minter_types::utils::{
    check_collection_creation_fee, generate_create_denom_msg, generate_multi_minter_mint_message,
    generate_update_denom_msg,
};
use pauser::PauseState;
use std::str::FromStr;

use crate::error::ContractError;
use crate::mint_instance::{
    get_mint_instance_by_id, return_latest_mint_instance_id, return_latest_mint_instance_id_in_use,
    MintInstance, MintInstanceParams, ACTIVE_MINT_INSTANCE_ID, MINT_INSTANCES,
    MINT_INSTANCE_IDS_IN_USE, MINT_INSTANCE_IDS_REMOVED,
};
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
    // Set the contract version
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    // Query factory parameters of instantiator
    // If the instantiator is not our factory, we won't be able to parse the response
    let _factory_params: ParamsResponse = deps.querier.query_wasm_smart(
        info.sender.clone().into_string(),
        &OpenEditionMinterFactoryQueryMsg::Params {},
    )?;

    // Retrieve the collection creation fee
    let collection_creation_fee: Coin = check_collection_creation_fee(deps.as_ref().querier)?;

    // Validate payment amount
    let amount = must_pay(&info.clone(), &collection_creation_fee.denom)?;
    // Exact amount must be paid
    if amount != collection_creation_fee.amount {
        return Err(ContractError::InvalidCreationFee {
            expected: vec![collection_creation_fee.clone()],
            sent: info.funds.clone(),
        });
    };
    let auth_details = msg.auth_details.clone();
    auth_details.validate(&deps.as_ref())?;

    // Set the pause state with the sender as the initial pauser
    let pause_state = PauseState::new()?;
    pause_state.set_pausers(
        deps.storage,
        info.sender.clone(),
        vec![auth_details.admin.clone()],
    )?;

    // Save collection and authorization details
    let collection_details = msg.collection_details.clone();
    COLLECTION.save(deps.storage, &collection_details)?;
    ACTIVE_MINT_INSTANCE_ID.save(deps.storage, &0)?;
    LAST_MINTED_TOKEN_ID.save(deps.storage, &0)?;
    AUTH_DETAILS.save(deps.storage, &auth_details)?;

    // Prepare and send the create denom message
    let nft_creation_fee = Coin {
        denom: collection_creation_fee.denom,
        amount: collection_creation_fee.amount,
    };
    let nft_creation_msg: CosmosMsg = generate_create_denom_msg(
        &collection_details,
        env.contract.address,
        nft_creation_fee,
        auth_details.payment_collector,
    )?
    .into();

    let res = Response::new()
        .add_message(nft_creation_msg)
        .add_attribute("action", "instantiate")
        .add_attribute("contract_version", CONTRACT_VERSION)
        .add_attribute("contract_name", CONTRACT_NAME)
        .add_attribute("mint_instance_id", "1");

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
        ExecuteMsg::Mint { mint_instance_id } => execute_mint(deps, env, info, mint_instance_id),
        ExecuteMsg::MintAdmin {
            recipient,
            mint_instance_id,
        } => execute_mint_admin(deps, env, info, recipient, mint_instance_id),
        ExecuteMsg::UpdateRoyaltyRatio {
            ratio,
            mint_instance_id,
        } => execute_update_royalty_ratio(deps, env, info, ratio, mint_instance_id),
        ExecuteMsg::UpdateMintPrice {
            mint_price,
            mint_instance_id,
        } => execute_update_mint_price(deps, env, info, mint_price, mint_instance_id),
        ExecuteMsg::UpdateWhitelistAddress {
            address,
            mint_instance_id,
        } => execute_update_whitelist_address(deps, env, info, address, mint_instance_id),
        ExecuteMsg::UpdateAdmin { admin } => execute_update_admin(deps, env, info, admin),
        ExecuteMsg::UpdatePaymentCollector { payment_collector } => {
            execute_update_payment_collector(deps, env, info, payment_collector)
        }
        ExecuteMsg::Pause {} => execute_pause(deps, env, info),
        ExecuteMsg::Unpause {} => execute_unpause(deps, env, info),
        ExecuteMsg::SetPausers { pausers } => execute_set_pausers(deps, env, info, pausers),
        ExecuteMsg::NewMintInstance {
            config,
            token_details,
        } => execute_new_mint_instance(deps, env, info, config, token_details),

        ExecuteMsg::UpdateRoyaltyReceivers { receivers } => {
            execute_update_royalty_receivers(deps, env, info, receivers)
        }
        ExecuteMsg::UpdateDenom {
            name,
            description,
            preview_uri,
        } => execute_update_denom(deps, env, info, name, description, preview_uri),
        ExecuteMsg::PurgeDenom {} => execute_purge_denom(deps, env, info),
        ExecuteMsg::RemoveMintInstance { mint_instance_id } => {
            execute_remove_mint_instance(deps, info, mint_instance_id)
        }
    }
}

pub fn execute_mint(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    mint_instance_id: Option<u32>,
) -> Result<Response, ContractError> {
    // Ensure contract is not paused
    let pause_state = PauseState::new()?;
    pause_state.error_if_paused(deps.storage)?;

    // Retrieve the mint_instance
    let (mint_instance_id, mut mint_instance) =
        get_mint_instance_by_id(mint_instance_id, deps.storage)?;

    let collection_details = COLLECTION.load(deps.storage)?;
    let config = mint_instance.clone().mint_instance_params.config;
    let token_details = mint_instance.clone().mint_instance_params.token_details;
    let mint_instance_minted_count = mint_instance.clone().minted_count;
    let auth_details = AUTH_DETAILS.load(deps.storage)?;

    // Check if any token limit is set and if it's reached
    if let Some(num_tokens) = config.num_tokens {
        if mint_instance_minted_count >= num_tokens {
            return Err(ContractError::NoTokensLeftToMint {});
        }
    }

    // Check if the end time is set and if it's passed
    if let Some(end_time) = config.end_time {
        if env.block.time > end_time {
            return Err(ContractError::PublicMintingEnded {});
        }
    };

    // Initialize user minting details
    let user_minting_details = UserMintingDetails::new(USER_MINTING_DETAILS_KEY);
    let mut user_details = user_minting_details
        .load(deps.storage, mint_instance_id, info.sender.clone())
        .unwrap_or_default();

    // Load and increment the minted count
    let last_token_id = LAST_MINTED_TOKEN_ID.load(deps.storage)?;
    let token_id = last_token_id + 1;
    LAST_MINTED_TOKEN_ID.save(deps.storage, &token_id)?;

    let mut mint_price = config.mint_price;

    // Check if minting is public
    let is_public = env.block.time >= config.start_time;

    let mut messages: Vec<CosmosMsg> = vec![];

    if !is_public {
        // Check if any whitelist is present
        if let Some(whitelist_address) = config.whitelist_address {
            // Check whitelist price
            let whitelist_price = check_whitelist_price(&whitelist_address, deps.as_ref())
                .map_err(|_| ContractError::WhitelistNotActive {})?;
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

    // Add the minted token to user details
    user_details.minted_tokens.push(Token {
        token_id: token_id.to_string(),
    });

    // Save the user details
    user_minting_details.save(
        deps.storage,
        mint_instance_id,
        info.sender.clone(),
        &user_details,
    );

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

    // Increment the mint_instance minted count and extract the mint_instance token id
    mint_instance.minted_count += 1;
    MINT_INSTANCES.save(deps.storage, mint_instance_id, &mint_instance)?;
    let mint_instance_token_id = mint_instance.minted_count;

    // Generate mint message
    let mint_msg: CosmosMsg = generate_multi_minter_mint_message(
        &collection_details,
        &token_details,
        token_id.to_string(),
        env.contract.address,
        info.sender,
        mint_instance_id.to_string(),
        mint_instance_token_id.to_string(),
    )?
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
        .add_attribute("mint_instance_token_id", mint_instance_token_id.to_string())
        .add_attribute("collection_id", collection_details.id)
        .add_attribute("mint_instance_id", mint_instance_id.to_string());

    Ok(res)
}

pub fn execute_mint_admin(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    recipient: String,
    mint_instance_id: Option<u32>,
) -> Result<Response, ContractError> {
    nonpayable(&info)?;

    // Find the mint_instance
    let (mint_instance_id, mut mint_instance) =
        get_mint_instance_by_id(mint_instance_id, deps.storage)?;

    let collection_details = COLLECTION.load(deps.storage)?;
    let token_details = mint_instance.clone().mint_instance_params.token_details;
    let config = mint_instance.clone().mint_instance_params.config;
    let auth_details = AUTH_DETAILS.load(deps.storage)?;
    // Check if admin
    if info.sender != auth_details.admin {
        return Err(ContractError::Unauthorized {});
    }
    // Check if token limit is set and if it is reached
    if let Some(num_tokens) = config.num_tokens {
        if mint_instance.minted_count >= num_tokens {
            return Err(ContractError::NoTokensLeftToMint {});
        }
    }

    // Check if end time is determined and if it is passed
    if let Some(end_time) = config.end_time {
        if env.block.time > end_time {
            return Err(ContractError::PublicMintingEnded {});
        }
    };

    let pause_state = PauseState::new()?;
    pause_state.error_if_paused(deps.storage)?;

    let recipient = deps.api.addr_validate(&recipient)?;

    let last_token_id = LAST_MINTED_TOKEN_ID.load(deps.storage)?;
    let token_id = last_token_id + 1;
    LAST_MINTED_TOKEN_ID.save(deps.storage, &token_id)?;

    let user_minting_details = UserMintingDetails::new(USER_MINTING_DETAILS_KEY);

    let mut user_details = user_minting_details
        .load(deps.storage, mint_instance_id, recipient.clone())
        .unwrap_or_default();
    // We are only updating these params but not checking the mint limit
    user_details.total_minted_count += 1;
    user_details.minted_tokens.push(Token {
        token_id: token_id.to_string(),
    });

    // Save the user details
    user_minting_details.save(
        deps.storage,
        mint_instance_id,
        recipient.clone(),
        &user_details,
    );

    // Increment the mint_instance minted count and extract the mint_instance token id
    mint_instance.minted_count += 1;
    MINT_INSTANCES.save(deps.storage, mint_instance_id, &mint_instance)?;
    let mint_instance_token_id = mint_instance.minted_count;

    let mint_msg: CosmosMsg = generate_multi_minter_mint_message(
        &collection_details,
        &token_details,
        token_id.to_string(),
        env.contract.address,
        recipient.clone(),
        mint_instance_id.to_string(),
        mint_instance_token_id.to_string(),
    )?
    .into();

    let res = Response::new()
        .add_message(mint_msg)
        .add_attribute("action", "mint")
        .add_attribute("token_id", token_id.to_string())
        .add_attribute("denom_id", collection_details.id)
        .add_attribute("mint_instance_token_id", mint_instance_token_id.to_string())
        .add_attribute("mint_instance_id", mint_instance_id.to_string());
    Ok(res)
}

pub fn execute_update_royalty_ratio(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    ratio: String,
    mint_instance_id: Option<u32>,
) -> Result<Response, ContractError> {
    let auth_details = AUTH_DETAILS.load(deps.storage)?;
    // Check if sender is admin
    if info.sender != auth_details.admin {
        return Err(ContractError::Unauthorized {});
    }
    // Find the mint_instance
    let (mint_instance_id, mut mint_instance) =
        get_mint_instance_by_id(mint_instance_id, deps.storage)?;
    // Extract the token details
    let mut mint_instance_token_details = mint_instance.clone().mint_instance_params.token_details;
    // Set the new ratio without checking the ratio
    mint_instance_token_details.royalty_ratio = Decimal::from_str(&ratio)?;
    // Check integrity of token details
    mint_instance_token_details.check_integrity()?;

    // Save the new token details
    mint_instance
        .mint_instance_params
        .token_details
        .royalty_ratio = Decimal::from_str(&ratio)?;
    MINT_INSTANCES.save(deps.storage, mint_instance_id, &mint_instance)?;

    let res = Response::new()
        .add_attribute("action", "update_royalty_ratio")
        .add_attribute("ratio", ratio.to_string())
        .add_attribute("mint_instance_id", mint_instance_id.to_string());
    Ok(res)
}

pub fn execute_update_mint_price(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    mint_price: Coin,
    mint_instance_id: Option<u32>,
) -> Result<Response, ContractError> {
    let auth_details = AUTH_DETAILS.load(deps.storage)?;

    // Check if sender is admin
    if info.sender != auth_details.admin {
        return Err(ContractError::Unauthorized {});
    }
    // Find the mint_instance
    let (mint_instance_id, mut mint_instance) =
        get_mint_instance_by_id(mint_instance_id, deps.storage)?;
    mint_instance.mint_instance_params.config.mint_price = mint_price.clone();

    MINT_INSTANCES.save(deps.storage, mint_instance_id, &mint_instance)?;

    let res = Response::new()
        .add_attribute("action", "update_mint_price")
        .add_attribute("mint_price_denom", mint_price.denom.to_string())
        .add_attribute("mint_price_amount", mint_price.amount.to_string())
        .add_attribute("mint_instance_id", mint_instance_id.to_string());
    Ok(res)
}

pub fn execute_update_admin(
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
    let mut auth_details = AUTH_DETAILS.load(deps.storage)?;
    // Check if sender is admin
    if info.sender != auth_details.admin {
        return Err(ContractError::Unauthorized {});
    }
    let validated_new_payment_collector = deps.api.addr_validate(&payment_collector)?;
    auth_details.payment_collector = validated_new_payment_collector.clone();
    AUTH_DETAILS.save(deps.storage, &auth_details)?;

    let res = Response::new()
        .add_attribute("action", "update_payment_collector")
        .add_attribute("payment_collector", payment_collector.to_string());
    Ok(res)
}

pub fn execute_update_whitelist_address(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    address: String,
    mint_instance_id: Option<u32>,
) -> Result<Response, ContractError> {
    // Check if sender is admin
    let auth_details = AUTH_DETAILS.load(deps.storage)?;
    if info.sender != auth_details.admin {
        return Err(ContractError::Unauthorized {});
    }
    // Find the mint_instance
    let (mint_instance_id, mut mint_instance) =
        get_mint_instance_by_id(mint_instance_id, deps.storage)?;

    let current_whitelist_address = mint_instance
        .mint_instance_params
        .config
        .whitelist_address
        .clone();

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
    mint_instance.mint_instance_params.config.whitelist_address =
        Some(validated_new_whitelist_address.clone());
    MINT_INSTANCES.save(deps.storage, mint_instance_id, &mint_instance)?;

    let res = Response::new()
        .add_attribute("action", "update_whitelist_address")
        .add_attribute("address", address.to_string())
        .add_attribute("mint_instance_id", mint_instance_id.to_string());
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

pub fn execute_new_mint_instance(
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
    // Check integrity of config
    config.check_integrity(env.block.time)?;
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
    let new_mint_instance_params = MintInstanceParams {
        config,
        token_details,
    };
    let new_mint_instance = MintInstance {
        minted_count: 0,
        mint_instance_params: new_mint_instance_params.clone(),
    };

    let latest_mint_instance_id = return_latest_mint_instance_id(deps.storage)?;
    let new_mint_instance_id = latest_mint_instance_id + 1;
    MINT_INSTANCES.save(deps.storage, new_mint_instance_id, &new_mint_instance)?;

    let mut mint_instance_ids_in_use = MINT_INSTANCE_IDS_IN_USE
        .load(deps.storage)
        .unwrap_or_default();
    mint_instance_ids_in_use.push(new_mint_instance_id);

    MINT_INSTANCE_IDS_IN_USE.save(deps.storage, &mint_instance_ids_in_use)?;
    ACTIVE_MINT_INSTANCE_ID.save(deps.storage, &new_mint_instance_id)?;

    let res = Response::new()
        .add_attribute("action", "new_mint_instance")
        .add_attribute("new_mint_instance_id", new_mint_instance_id.to_string());

    Ok(res)
}

pub fn execute_remove_mint_instance(
    deps: DepsMut,
    info: MessageInfo,
    mint_instance_id: u32,
) -> Result<Response, ContractError> {
    let auth_details = AUTH_DETAILS.load(deps.storage)?;
    // Check if sender is admin
    if info.sender != auth_details.admin {
        return Err(ContractError::Unauthorized {});
    }

    // If active mint_instance id is 0 then no mint_instance is available
    if mint_instance_id == 0 {
        return Err(ContractError::NoMintInstanceAvailable {});
    }
    let (mint_instance_id, mint_instance) =
        get_mint_instance_by_id(Some(mint_instance_id), deps.storage)?;

    if mint_instance.minted_count > 0 {
        return Err(ContractError::MintInstanceCantBeRemoved {});
    }
    MINT_INSTANCES.remove(deps.storage, mint_instance_id);
    let mut mint_instance_ids_in_use = MINT_INSTANCE_IDS_IN_USE.load(deps.storage)?;

    // Remove the mint_instance id from the list
    mint_instance_ids_in_use.retain(|&x| x != mint_instance_id);
    MINT_INSTANCE_IDS_IN_USE.save(deps.storage, &mint_instance_ids_in_use)?;

    let mut mint_instance_ids_removed = MINT_INSTANCE_IDS_REMOVED
        .load(deps.storage)
        .unwrap_or_default();

    // Add the mint_instance id to the removed list
    mint_instance_ids_removed.push(mint_instance_id);
    MINT_INSTANCE_IDS_REMOVED.save(deps.storage, &mint_instance_ids_removed)?;

    let new_active_mint_instance_id = return_latest_mint_instance_id_in_use(deps.storage)?;
    ACTIVE_MINT_INSTANCE_ID.save(deps.storage, &new_active_mint_instance_id)?;

    Ok(Response::new()
        .add_attribute("action", "remove_mint_instance")
        .add_attribute(
            "new_active_mint_instance_id",
            new_active_mint_instance_id.to_string(),
        )
        .add_attribute("removed_mint_instance_id", mint_instance_id.to_string()))
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
    // Current mint_instances admin can update the denom
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
        BaseMinterQueryMsg::MintHistory { address } => {
            to_json_binary(&query_mint_history(deps, env, address, None)?)
        }
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
            QueryMsgExtension::ActiveMintInstanceId {} => {
                to_json_binary(&query_active_mint_instance_id(deps, env)?)
            }
            QueryMsgExtension::AllMintInstances {} => {
                to_json_binary(&query_all_mint_instances(deps, env)?)
            }
            QueryMsgExtension::Config { mint_instance_id } => {
                to_json_binary(&query_config(deps, env, mint_instance_id)?)
            }
            QueryMsgExtension::UserMintingDetails {
                address,
                mint_instance_id,
            } => to_json_binary(&query_user_minting_details(
                deps,
                env,
                address,
                mint_instance_id,
            )?),
            QueryMsgExtension::TokensRemainingInMintInstance { mint_instance_id } => {
                to_json_binary(&query_tokens_remaining_in_mint_instance(
                    deps,
                    env,
                    mint_instance_id,
                )?)
            }
            QueryMsgExtension::TokenDetails { mint_instance_id } => {
                to_json_binary(&query_token_details(deps, env, mint_instance_id)?)
            }
            QueryMsgExtension::TokensMintedInMintInstance { mint_instance_id } => to_json_binary(
                &query_tokens_minted_in_mint_instance(deps, env, mint_instance_id)?,
            ),
            QueryMsgExtension::MintHistory {
                address,
                mint_instance_id,
            } => to_json_binary(&query_mint_history(deps, env, address, mint_instance_id)?),
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
    mint_instance_id: Option<u32>,
) -> Result<TokenDetails, ContractError> {
    // Find the mint_instance
    let (_, mint_instance) = get_mint_instance_by_id(mint_instance_id, deps.storage)?;
    let token_details = mint_instance.mint_instance_params.token_details;
    Ok(token_details)
}

fn query_config(
    deps: Deps,
    _env: Env,
    mint_instance_id: Option<u32>,
) -> Result<Config, ContractError> {
    // Find the mint_instance
    let (_, mint_instance) = get_mint_instance_by_id(mint_instance_id, deps.storage)?;
    let config = mint_instance.mint_instance_params.config;
    Ok(config)
}

fn query_user_minting_details(
    deps: Deps,
    _env: Env,
    address: String,
    mint_instance_id: Option<u32>,
) -> Result<UserDetails, ContractError> {
    let address = deps.api.addr_validate(&address)?;
    let mint_instance_id = mint_instance_id.unwrap_or(ACTIVE_MINT_INSTANCE_ID.load(deps.storage)?);
    let user_minting_details = UserMintingDetails::new(USER_MINTING_DETAILS_KEY);
    let user_details = user_minting_details
        .load(deps.storage, mint_instance_id, address)
        .unwrap_or_default();
    Ok(user_details)
}

fn query_total_tokens_minted(deps: Deps, _env: Env) -> Result<u32, ContractError> {
    let total_minted_count = LAST_MINTED_TOKEN_ID.load(deps.storage)?;
    Ok(total_minted_count)
}

fn query_tokens_remaining_in_mint_instance(
    deps: Deps,
    _env: Env,
    mint_instance_id: Option<u32>,
) -> Result<u32, ContractError> {
    let (_, mint_instance) = get_mint_instance_by_id(mint_instance_id, deps.storage)?;
    let config = mint_instance.mint_instance_params.config;
    let mint_instance_minted_count = mint_instance.minted_count;

    if let Some(num_tokens) = config.num_tokens {
        let tokens_remaining = num_tokens - mint_instance_minted_count;
        Ok(tokens_remaining)
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

fn query_active_mint_instance_id(deps: Deps, _env: Env) -> Result<u32, ContractError> {
    let active_mint_instance_id = ACTIVE_MINT_INSTANCE_ID.load(deps.storage).unwrap_or(0);
    Ok(active_mint_instance_id)
}

fn query_all_mint_instances(
    deps: Deps,
    _env: Env,
) -> Result<Vec<(u32, MintInstance)>, ContractError> {
    let mint_instance_ids_in_use = MINT_INSTANCE_IDS_IN_USE
        .load(deps.storage)
        .unwrap_or_default();

    let mut mint_instances: Vec<(u32, MintInstance)> = vec![];

    if mint_instance_ids_in_use.is_empty() {
        return Ok(mint_instances);
    }

    for mint_instance_id in mint_instance_ids_in_use {
        let (_, mint_instance) = get_mint_instance_by_id(Some(mint_instance_id), deps.storage)?;
        mint_instances.push((mint_instance_id, mint_instance));
    }
    Ok(mint_instances)
}

fn query_auth_details(deps: Deps, _env: Env) -> Result<AuthDetails, ContractError> {
    let auth_details = AUTH_DETAILS.load(deps.storage)?;
    Ok(auth_details)
}

fn query_tokens_minted_in_mint_instance(
    deps: Deps,
    _env: Env,
    mint_instance_id: Option<u32>,
) -> Result<u32, ContractError> {
    let (_, mint_instance) = get_mint_instance_by_id(mint_instance_id, deps.storage)?;
    let mint_instance_minted_count = mint_instance.minted_count;
    Ok(mint_instance_minted_count)
}

fn query_mint_history(
    deps: Deps,
    _env: Env,
    address: String,
    mint_instance_id: Option<u32>,
) -> Result<MintHistoryResponse, ContractError> {
    let address = deps.api.addr_validate(&address)?;
    let mint_instance_id = mint_instance_id.unwrap_or(ACTIVE_MINT_INSTANCE_ID.load(deps.storage)?);
    if mint_instance_id == 0 {
        return Err(ContractError::NoMintInstanceAvailable {});
    }
    let user_minting_details = UserMintingDetails::new(USER_MINTING_DETAILS_KEY);
    let user_details = user_minting_details
        .load(deps.storage, mint_instance_id, address)
        .unwrap_or_default();
    let public_minted_count = user_details.public_mint_count;
    let total_minted_count = user_details.total_minted_count;
    let public_mint_limit = MINT_INSTANCES
        .load(deps.storage, mint_instance_id)?
        .mint_instance_params
        .config
        .per_address_limit
        .unwrap_or(0);
    let mint_history = MintHistoryResponse {
        public_minted_count,
        total_minted_count,
        public_mint_limit,
    };
    Ok(mint_history)
}
