#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, to_json_binary, wasm_execute, Binary, Coin, Deps, DepsMut, Env, MessageInfo, Order,
    Response, StdResult, Timestamp, Uint128,
};
use cw2::set_contract_version;
use cw_storage_plus::Bound;
use cw_utils::{maybe_addr, must_pay};
// use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, UpdateWhitelistRound};
use crate::state::{CONFIG, MEMBERS};
use types::whitelist::{
    Config, HasEndedResponse, HasMemberResponse, HasStartedResponse, IsActiveResponse,
    MembersResponse, PerAddressLimitResponse, WhitelistQueryMsgs,
};
const CONTRACT_NAME: &str = "crates.io:omniflix-whitelist";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const PAGINATION_LIMIT: u32 = 100;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let admin = match msg.admin {
        Some(admin) => deps.api.addr_validate(&admin)?,
        None => info.sender,
    };
    // Check member limit
    if msg.member_limit <= 0 {
        return Err(ContractError::InvalidMemberLimit {});
    }

    // Check start time
    if msg.start_time < env.block.time || msg.end_time < msg.start_time {
        return Err(ContractError::InvalidStartTime {});
    }

    // Check per address limit
    if msg.per_address_limit <= 0 {
        return Err(ContractError::InvalidPerAddressLimit {});
    }
    // Check mint price
    // Probably not needed
    if msg.mint_price.amount < Uint128::zero() {
        return Err(ContractError::InvalidMintPrice {});
    }

    // Remove duplicates
    let mut unvalidated_members = msg.members;
    unvalidated_members.sort_unstable();
    unvalidated_members.dedup();

    // Check if final limit is valid
    if unvalidated_members.len() > msg.member_limit as usize {
        return Err(ContractError::InvalidMemberLimit {});
    }
    for member in unvalidated_members {
        MEMBERS.save(deps.storage, deps.api.addr_validate(&member)?, &true)?;
    }

    // Save config
    let config = Config {
        admin,
        start_time: msg.start_time,
        end_time: msg.end_time,
        mint_price: msg.mint_price,
        per_address_limit: msg.per_address_limit,
        member_limit: msg.member_limit,
        is_frozen: false,
    };
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::default()
        .add_attribute("method", "instantiate")
        .add_attribute("admin", config.admin)
        .add_attribute("start_time", config.start_time.to_string())
        .add_attribute("end_time", config.end_time.to_string())
        .add_attribute("mint_price", config.mint_price.to_string())
        .add_attribute("per_address_limit", config.per_address_limit.to_string())
        .add_attribute("member_limit", config.member_limit.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::UpdateStartTime {
            start_time,
            minter_address,
        } => update_start_time(deps, env, info, start_time, minter_address),
        ExecuteMsg::UpdateEndTime {
            end_time,
            minter_address,
        } => update_end_time(deps, env, info, end_time, minter_address),
        ExecuteMsg::AddMembers { addresses } => add_members(deps, env, info, addresses),
        ExecuteMsg::RemoveMembers { addresses } => remove_members(deps, env, info, addresses),
        ExecuteMsg::UpdatePerAddressLimit {
            amount,
            minter_address,
        } => update_per_address_limit(deps, env, info, amount, minter_address),
        ExecuteMsg::IncreaseMemberLimit { amount } => {
            increase_member_limit(deps, env, info, amount)
        }
        ExecuteMsg::UpdateAdmin { admin } => update_admin(deps, env, info, admin),
        ExecuteMsg::Freeze {} => freeze(deps, env, info),
        ExecuteMsg::UpdateMintPrice {
            mint_price,
            minter_address,
        } => update_mint_price(deps, env, info, mint_price, minter_address),
    }
}

pub fn update_start_time(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    start_time: Timestamp,
    minter_address: Option<String>,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;

    // Check if frozen
    if config.is_frozen {
        return Err(ContractError::WhitelistFrozen {});
    }
    // Check if sender is admin
    if info.sender != config.admin {
        return Err(ContractError::Unauthorized {});
    }
    // Check if whitelist already started
    if env.block.time > config.start_time {
        return Err(ContractError::WhiteListAlreadyStarted {});
    }

    if start_time < env.block.time || start_time > config.end_time {
        return Err(ContractError::InvalidStartTime {});
    }
    config.start_time = start_time;

    if minter_address.is_some() {
        let addr = deps.api.addr_validate(&minter_address.unwrap())?;
        // Generate minter message
        let update_minter_msg = UpdateWhitelistRound {
            start_time: Some(start_time),
            end_time: None,
            mint_price: None,
            round_limit: None,
        };
        let msg = to_json_binary(&update_minter_msg)?;
        // Send message to minter contract
        wasm_execute(addr, &msg, Vec::new())?;
    }

    CONFIG.save(deps.storage, &config)?;
    Ok(Response::default()
        .add_attribute("method", "update_start_time")
        .add_attribute("start_time", config.start_time.to_string()))
}

pub fn update_end_time(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    end_time: Timestamp,
    minter_address: Option<String>,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;

    if info.sender != config.admin {
        return Err(ContractError::Unauthorized {});
    }

    // Check if frozen
    if config.is_frozen {
        return Err(ContractError::WhitelistFrozen {});
    }
    // We can let creator to update end time even after whitelist started
    // Most logical approach is to let creator to update end time but only to extend it
    if env.block.time > config.start_time && end_time < config.end_time {
        return Err(ContractError::WhiteListAlreadyStarted {});
    }
    // Check if end time is valid
    if end_time < config.start_time || end_time < env.block.time {
        return Err(ContractError::InvalidEndTime {});
    }
    config.end_time = end_time;
    if minter_address.is_some() {
        let addr = deps.api.addr_validate(&minter_address.unwrap())?;
        // Generate minter message
        let update_minter_msg = UpdateWhitelistRound {
            start_time: None,
            end_time: Some(end_time),
            mint_price: None,
            round_limit: None,
        };
        let msg = to_json_binary(&update_minter_msg)?;
        // Send message to minter contract
        wasm_execute(addr, &msg, Vec::new())?;
    }

    CONFIG.save(deps.storage, &config)?;
    Ok(Response::default()
        .add_attribute("method", "update_end_time")
        .add_attribute("end_time", config.end_time.to_string()))
}

pub fn add_members(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    addresses: Vec<String>,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    if info.sender != config.admin {
        return Err(ContractError::Unauthorized {});
    }
    if env.block.time > config.end_time {
        return Err(ContractError::WhitelistEnded {});
    }

    // Check if frozen
    if config.is_frozen {
        return Err(ContractError::WhitelistFrozen {});
    }

    // Remove duplicates
    let mut unvalidated_members = addresses;
    unvalidated_members.sort_unstable();
    unvalidated_members.dedup();

    // Check if address is already in whitelist
    for member in &unvalidated_members {
        if MEMBERS.may_load(deps.storage, deps.api.addr_validate(&member)?)? == Some(true) {
        } else {
            MEMBERS.save(deps.storage, deps.api.addr_validate(&member)?, &true)?;
        }
    }

    // Check if final list is longer than member limit

    let member_count: u32 = MEMBERS
        .range(deps.storage, None, None, Order::Ascending)
        .count() as u32;

    if member_count > config.member_limit as u32 {
        return Err(ContractError::MemberLimitReached {
            member_limit: config.member_limit,
            current_member_count: member_count,
        });
    }

    Ok(Response::default()
        .add_attribute("method", "add_members")
        .add_attribute("members", unvalidated_members.join(",")))
}

pub fn remove_members(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    addresses: Vec<String>,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;

    if info.sender != config.admin {
        return Err(ContractError::Unauthorized {});
    }
    // Check if whitelist started
    if env.block.time > config.start_time {
        return Err(ContractError::WhiteListAlreadyStarted {});
    }

    // Check if frozen
    if config.is_frozen {
        return Err(ContractError::WhitelistFrozen {});
    }
    // Remove duplicates
    let mut unvalidated_members = addresses;
    unvalidated_members.sort_unstable();
    unvalidated_members.dedup();

    // Check if address is already in whitelist
    for member in &unvalidated_members {
        if MEMBERS.may_load(deps.storage, deps.api.addr_validate(&member)?)? == Some(true) {
            // When removing member we should not ignore it
            MEMBERS.remove(deps.storage, deps.api.addr_validate(&member)?);
        } else {
            return Err(ContractError::MemberDoesNotExist {
                member: member.to_string(),
            });
        }
    }

    Ok(Response::default()
        .add_attribute("method", "remove_members")
        .add_attribute("members removed", unvalidated_members.join(",")))
}

pub fn update_per_address_limit(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: u32,
    minter_address: Option<String>,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;

    if info.sender != config.admin {
        return Err(ContractError::Unauthorized {});
    }
    // Check if frozen
    if config.is_frozen {
        return Err(ContractError::WhitelistFrozen {});
    }
    if env.block.time > config.end_time {
        return Err(ContractError::WhitelistEnded {});
    }
    // This execution should be allowed only when whitelist is not started
    // TODO correct this
    if env.block.time > config.start_time {
        return Err(ContractError::WhiteListAlreadyStarted {});
    }

    if amount <= 0 {
        return Err(ContractError::InvalidPerAddressLimit {});
    }
    if minter_address.is_some() {
        let addr = deps.api.addr_validate(&minter_address.unwrap())?;
        // Generate minter message
        let update_minter_msg = UpdateWhitelistRound {
            start_time: None,
            end_time: None,
            mint_price: None,
            round_limit: Some(amount),
        };
        let msg = to_json_binary(&update_minter_msg)?;
        // Send message to minter contract
        wasm_execute(addr, &msg, Vec::new())?;
    }

    config.per_address_limit = amount;

    CONFIG.save(deps.storage, &config)?;
    Ok(Response::default()
        .add_attribute("method", "update_per_address_limit")
        .add_attribute("per_address_limit", config.per_address_limit.to_string()))
}

pub fn increase_member_limit(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: u32,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;

    if info.sender != config.admin {
        return Err(ContractError::Unauthorized {});
    }
    // Check if frozen
    if config.is_frozen {
        return Err(ContractError::WhitelistFrozen {});
    }
    if env.block.time > config.end_time {
        return Err(ContractError::WhitelistEnded {});
    }

    if amount <= 0 {
        return Err(ContractError::InvalidMemberLimit {});
    }
    config.member_limit += amount;

    CONFIG.save(deps.storage, &config)?;
    Ok(Response::default()
        .add_attribute("method", "increase_member_limit")
        .add_attribute("member_limit", config.member_limit.to_string()))
}

pub fn update_admin(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    admin: String,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;

    if info.sender != config.admin {
        return Err(ContractError::Unauthorized {});
    }
    // Check if frozen
    if config.is_frozen {
        return Err(ContractError::WhitelistFrozen {});
    }
    // Check if whitelist ended
    if env.block.time > config.end_time {
        return Err(ContractError::WhitelistEnded {});
    }

    config.admin = deps.api.addr_validate(&admin)?;

    CONFIG.save(deps.storage, &config)?;
    Ok(Response::default()
        .add_attribute("method", "update_admin")
        .add_attribute("admin", config.admin))
}

pub fn freeze(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;

    if info.sender != config.admin {
        return Err(ContractError::Unauthorized {});
    }
    // Check if frozen
    if config.is_frozen {
        return Err(ContractError::WhitelistFrozen {});
    }
    if env.block.time > config.end_time {
        return Err(ContractError::WhitelistEnded {});
    }

    config.is_frozen = true;

    CONFIG.save(deps.storage, &config)?;
    Ok(Response::default()
        .add_attribute("method", "freeze")
        .add_attribute("end_time", config.end_time.to_string()))
}

pub fn update_mint_price(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    mint_price: Coin,
    minter_address: Option<String>,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;

    if info.sender != config.admin {
        return Err(ContractError::Unauthorized {});
    }
    // Check if frozen
    if config.is_frozen {
        return Err(ContractError::WhitelistFrozen {});
    }
    if env.block.time > config.end_time {
        return Err(ContractError::WhitelistEnded {});
    }

    if mint_price.amount < Uint128::zero() {
        return Err(ContractError::InvalidMintPrice {});
    }
    config.mint_price = mint_price.clone();

    if minter_address.is_some() {
        let addr = deps.api.addr_validate(&minter_address.unwrap())?;
        // Generate minter message
        let update_minter_msg = UpdateWhitelistRound {
            start_time: None,
            end_time: None,
            mint_price: Some(mint_price.amount),
            round_limit: None,
        };
        let msg = to_json_binary(&update_minter_msg)?;
        // Send message to minter contract
        wasm_execute(addr, &msg, Vec::new())?;
    }

    CONFIG.save(deps.storage, &config)?;
    Ok(Response::default()
        .add_attribute("method", "update_mint_price")
        .add_attribute("mint_price", config.mint_price.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: WhitelistQueryMsgs) -> StdResult<Binary> {
    match msg {
        WhitelistQueryMsgs::HasStarted {} => to_json_binary(&query_has_started(deps, env)?),
        WhitelistQueryMsgs::HasEnded {} => to_json_binary(&query_has_ended(deps, env)?),
        WhitelistQueryMsgs::IsActive {} => to_json_binary(&query_is_active(deps, env)?),
        WhitelistQueryMsgs::Members { start_after, limit } => {
            to_json_binary(&query_members(deps, env, start_after, limit)?)
        }
        WhitelistQueryMsgs::HasMember { member } => {
            to_json_binary(&query_has_member(deps, env, member)?)
        }
        WhitelistQueryMsgs::Config {} => to_json_binary(&query_config(deps, env)?),
        WhitelistQueryMsgs::PerAddressLimit {} => {
            to_json_binary(&query_per_address_limit(deps, env)?)
        }
    }
}

pub fn query_per_address_limit(deps: Deps, _env: Env) -> StdResult<PerAddressLimitResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(PerAddressLimitResponse {
        per_address_limit: config.per_address_limit,
    })
}

pub fn query_has_started(deps: Deps, env: Env) -> StdResult<HasStartedResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(HasStartedResponse {
        has_started: env.block.time > config.start_time,
    })
}

pub fn query_has_ended(deps: Deps, env: Env) -> StdResult<HasEndedResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(HasEndedResponse {
        has_ended: env.block.time > config.end_time,
    })
}

pub fn query_is_active(deps: Deps, env: Env) -> StdResult<IsActiveResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(IsActiveResponse {
        is_active: env.block.time > config.start_time && env.block.time < config.end_time,
    })
}

pub fn query_members(
    deps: Deps,
    _env: Env,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<MembersResponse> {
    let start_addr = maybe_addr(deps.api, start_after)?;
    let start = start_addr.map(Bound::exclusive);

    let limit = limit.unwrap_or(PAGINATION_LIMIT);

    let members: Vec<String> = MEMBERS
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit as usize)
        .map(|item| {
            let (k, _) = item?;
            Ok(k.to_string())
        })
        .collect::<StdResult<Vec<String>>>()?;

    Ok(MembersResponse { members })
}

pub fn query_has_member(deps: Deps, _env: Env, member: String) -> StdResult<HasMemberResponse> {
    let address = deps.api.addr_validate(&member)?;
    let has_member = MEMBERS.may_load(deps.storage, address)?.unwrap_or(false);
    Ok(HasMemberResponse { has_member })
}

pub fn query_config(deps: Deps, _env: Env) -> StdResult<Config> {
    let config = CONFIG.load(deps.storage)?;
    Ok(config)
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::{
        testing::{mock_dependencies, mock_env, mock_info},
        Coin,
    };

    use super::*;

    pub fn return_inst_message() -> InstantiateMsg {
        let members: Vec<String> = (0..100).map(|i| format!("addr{}", i)).collect();

        let msg = InstantiateMsg {
            admin: None,
            start_time: Timestamp::from_seconds(1_000_000),
            end_time: Timestamp::from_seconds(5_000_000),
            mint_price: Coin {
                denom: "uflix".to_string(),
                amount: Uint128::from(1u128),
            },
            per_address_limit: 1,
            members: members,
            member_limit: 200,
        };
        msg
    }

    #[test]
    fn proper_instantiation() {
        let mut deps = mock_dependencies();
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(100_000);
        let info = mock_info("creator", &[]);
        let mut msg = return_inst_message();

        // Send invalid member limit
        msg.member_limit = 0;
        let res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg.clone()).unwrap_err();
        assert_eq!(res, ContractError::InvalidMemberLimit {});

        // Send invalid start time
        let mut msg = return_inst_message();
        msg.start_time = Timestamp::from_seconds(100_000 - 1);
        let res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg.clone()).unwrap_err();
        assert_eq!(res, ContractError::InvalidStartTime {});

        // Send duplicate members
        let mut msg = return_inst_message();
        msg.members.push("addr0".to_string());
        msg.members.push("addr0".to_string());
        msg.members.push("addr0".to_string());
        msg.members.push("addr0".to_string());
        msg.members.push("addr0".to_string());
        // We are sending 101 members now
        let res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg.clone()).unwrap();

        // Check if members are saved
        let members = query_members(deps.as_ref(), env.clone(), None, Some(150))
            .unwrap()
            .members;
        assert_eq!(members.len(), 100);
        // Find how many addr0 are there
        let mut addr0_count = 0;
        for member in members {
            if member == "addr0" {
                addr0_count += 1;
            }
        }
        assert_eq!(addr0_count, 1);

        // Check config
        let config = query_config(deps.as_ref(), env.clone()).unwrap();
        assert_eq!(config.member_limit, 200);
        assert_eq!(config.per_address_limit, 1);
        assert_eq!(config.start_time, Timestamp::from_seconds(1_000_000));
        assert_eq!(config.end_time, Timestamp::from_seconds(5_000_000));
        assert_eq!(config.mint_price, Coin::new(1, "uflix"));
        assert_eq!(config.is_frozen, false);
    }

    #[test]
    fn test_update_start_time() {
        let mut deps = mock_dependencies();
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(100_000);
        let info = mock_info("creator", &[]);
        let mut msg = return_inst_message();

        // instantiate
        let res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg.clone()).unwrap();

        // Try updating already started whitelist
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(1_000_001);
        let info = mock_info("creator", &[]);
        let res = update_start_time(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            Timestamp::from_seconds(1_000_002),
            None,
        )
        .unwrap_err();
        assert_eq!(res, ContractError::WhiteListAlreadyStarted {});

        // Try updating with invalid start time
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(100_000);
        let info = mock_info("creator", &[]);
        let res = update_start_time(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            Timestamp::from_seconds(100_000 - 1),
            None,
        )
        .unwrap_err();
        assert_eq!(res, ContractError::InvalidStartTime {});

        // Try updating with valid start time
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(100_000);
        let info = mock_info("creator", &[]);
        let res = update_start_time(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            Timestamp::from_seconds(100_000 + 1),
            None,
        )
        .unwrap();
    }

    #[test]
    fn test_update_end_time() {
        let mut deps = mock_dependencies();
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(100_000);
        let info = mock_info("creator", &[]);
        let mut msg = return_inst_message();

        // instantiate
        let res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg.clone()).unwrap();

        // Try updating with invalid end time
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(100_000);
        let info = mock_info("creator", &[]);
        let res = update_end_time(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            Timestamp::from_seconds(100_000 - 1),
            None,
        )
        .unwrap_err();
        assert_eq!(res, ContractError::InvalidEndTime {});

        // Try updating end time after whitelist started
        // This should fail if end time is less than current end time
        // You can only extend end time if whitelist is already started
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(1_000_001);
        let info = mock_info("creator", &[]);
        let res = update_end_time(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            Timestamp::from_seconds(5_000_000 - 1),
            None,
        )
        .unwrap_err();
        assert_eq!(res, ContractError::WhiteListAlreadyStarted {});

        // Try updating end time after whitelist started
        // Extend end time
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(1_000_001);
        let info = mock_info("creator", &[]);
        let res = update_end_time(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            Timestamp::from_seconds(5_000_000 + 1),
            None,
        )
        .unwrap();
    }

    #[test]
    fn test_add_members() {
        let mut deps = mock_dependencies();
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(100_000);
        let info = mock_info("creator", &[]);
        let mut msg = return_inst_message();

        // instantiate
        let res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg.clone()).unwrap();

        // Try adding members after whitelist ended
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(5_000_001);
        let info = mock_info("creator", &[]);
        let res = add_members(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            vec!["addr101".to_string()],
        )
        .unwrap_err();
        assert_eq!(res, ContractError::WhitelistEnded {});

        // Try adding duplicate members
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(100_000);
        let info = mock_info("creator", &[]);
        let res = add_members(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            vec!["addr0".to_string(), "addr0".to_string()],
        )
        .unwrap();

        // Check if members are saved
        let members = query_members(deps.as_ref(), env.clone(), None, Some(150))
            .unwrap()
            .members;
        assert_eq!(members.len(), 100);

        // Try adding diffirent members but with a list of duplicates
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(100_000);
        let info = mock_info("creator", &[]);

        let mut members = vec!["addr101".to_string(), "addr101".to_string()];
        let res = add_members(deps.as_mut(), env.clone(), info.clone(), members.clone()).unwrap();

        // Check if members are saved
        let members = query_members(deps.as_ref(), env.clone(), None, Some(150))
            .unwrap()
            .members;
        assert_eq!(members.len(), 101);

        // Try adding members more than member limit
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(100_000);
        let info = mock_info("creator", &[]);
        let mut members = vec![];
        for i in 100..201 {
            members.push(format!("addr{}", i));
        }
        let res =
            add_members(deps.as_mut(), env.clone(), info.clone(), members.clone()).unwrap_err();
        assert_eq!(
            res,
            ContractError::MemberLimitReached {
                member_limit: 200,
                current_member_count: 201
            }
        );
    }

    #[test]
    fn test_remove_members() {
        let mut deps = mock_dependencies();
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(100_000);
        let info = mock_info("creator", &[]);
        let mut msg = return_inst_message();

        // instantiate
        let res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg.clone()).unwrap();

        // Try removing members after whitelist started
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(1_000_001);
        let info = mock_info("creator", &[]);
        let res = remove_members(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            vec!["addr0".to_string()],
        )
        .unwrap_err();
        assert_eq!(res, ContractError::WhiteListAlreadyStarted {});

        // Try removing members who are not in whitelist
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(100_000);
        let info = mock_info("creator", &[]);
        // Try removing members who are not in whitelist
        let res = remove_members(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            vec!["addr101".to_string()],
        )
        .unwrap_err();
        assert_eq!(
            res,
            ContractError::MemberDoesNotExist {
                member: "addr101".to_string()
            }
        );

        // Try removing members with a list of duplicates
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(100_000);
        let info = mock_info("creator", &[]);
        let res = remove_members(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            vec!["addr0".to_string(), "addr0".to_string()],
        )
        .unwrap();

        // Check if member is removed
        let members = query_members(deps.as_ref(), env.clone(), None, Some(150))
            .unwrap()
            .members;
        assert_eq!(members.contains(&"addr0".to_string()), false);
        assert_eq!(members.len(), 99);
    }

    #[test]
    fn test_update_per_address_limit() {
        let mut deps = mock_dependencies();
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(100_000);
        let info = mock_info("creator", &[]);
        let mut msg = return_inst_message();

        // instantiate
        let res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg.clone()).unwrap();

        // Try updating per address limit after whitelist started
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(1_000_001);
        let info = mock_info("creator", &[]);
        let res = update_per_address_limit(deps.as_mut(), env.clone(), info.clone(), 2, None)
            .unwrap_err();
        assert_eq!(res, ContractError::WhiteListAlreadyStarted {});

        // Try updating per address limit after whitelist ended
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(5_000_001);
        let info = mock_info("creator", &[]);
        let res = update_per_address_limit(deps.as_mut(), env.clone(), info.clone(), 2, None)
            .unwrap_err();
        assert_eq!(res, ContractError::WhitelistEnded {});

        // Try updating per address limit with invalid amount
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(100_000);
        let info = mock_info("creator", &[]);
        let res = update_per_address_limit(deps.as_mut(), env.clone(), info.clone(), 0, None)
            .unwrap_err();
        assert_eq!(res, ContractError::InvalidPerAddressLimit {});

        // Try updating per address limit with valid amount
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(100_000);
        let info = mock_info("creator", &[]);
        let res =
            update_per_address_limit(deps.as_mut(), env.clone(), info.clone(), 2, None).unwrap();
    }

    #[test]
    fn test_increase_member_limit() {
        let mut deps = mock_dependencies();
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(100_000);

        let info = mock_info("creator", &[]);
        let mut msg = return_inst_message();

        // instantiate
        let res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg.clone()).unwrap();

        // Try increasing member limit after whitelist ended
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(5_000_001);
        let info = mock_info("creator", &[]);
        let res = increase_member_limit(deps.as_mut(), env.clone(), info.clone(), 2).unwrap_err();
        assert_eq!(res, ContractError::WhitelistEnded {});

        // Try increasing member limit with invalid amount
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(100_000);
        let info = mock_info("creator", &[]);
        // Try increasing member limit with invalid amount
        let res = increase_member_limit(deps.as_mut(), env.clone(), info.clone(), 0).unwrap_err();
        assert_eq!(res, ContractError::InvalidMemberLimit {});

        // Try increasing member limit with valid amount
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(100_000);
        let info = mock_info("creator", &[]);

        let res = increase_member_limit(deps.as_mut(), env.clone(), info.clone(), 2).unwrap();
    }

    #[test]
    fn test_update_admin() {
        let mut deps = mock_dependencies();
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(100_000);

        let info = mock_info("creator", &[]);
        let mut msg = return_inst_message();

        // instantiate
        let res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg.clone()).unwrap();

        // Try updating admin after whitelist ended
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(5_000_001);
        let info = mock_info("creator", &[]);
        let res = update_admin(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            "addr101".to_string(),
        )
        .unwrap_err();
        assert_eq!(res, ContractError::WhitelistEnded {});

        // Try updating admin without admin permission
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(100_000);
        let info = mock_info("non_admin", &[]);
        let res = update_admin(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            "addr101".to_string(),
        )
        .unwrap_err();
        assert_eq!(res, ContractError::Unauthorized {});

        // Try updating admin with valid address
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(100_000);
        let info = mock_info("creator", &[]);

        let res = update_admin(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            "addr101".to_string(),
        )
        .unwrap();
    }

    #[test]
    fn test_freeze() {
        let mut deps = mock_dependencies();
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(100_000);

        let info = mock_info("creator", &[]);
        let mut msg = return_inst_message();

        // instantiate
        let res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg.clone()).unwrap();

        // Try freezing after whitelist ended
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(5_000_001);
        let info = mock_info("creator", &[]);
        let res = freeze(deps.as_mut(), env.clone(), info.clone()).unwrap_err();
        assert_eq!(res, ContractError::WhitelistEnded {});

        // Try freezing without admin permission
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(100_000);
        let info = mock_info("non_admin", &[]);
        let res = freeze(deps.as_mut(), env.clone(), info.clone()).unwrap_err();
        assert_eq!(res, ContractError::Unauthorized {});

        // Try freezing with valid address
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(100_000);
        let info = mock_info("creator", &[]);

        let res = freeze(deps.as_mut(), env.clone(), info.clone()).unwrap();

        // Try every other function after freeze
        // Every function should fail after freeze

        // Try updating start time after freeze
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(100_000);
        let info = mock_info("creator", &[]);
        let res = update_start_time(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            Timestamp::from_seconds(100_000 + 1),
            None,
        )
        .unwrap_err();
        assert_eq!(res, ContractError::WhitelistFrozen {});

        // Try updating end time after freeze
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(100_000);
        let info = mock_info("creator", &[]);
        let res = update_end_time(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            Timestamp::from_seconds(5_000_000 + 1),
            None,
        )
        .unwrap_err();
        assert_eq!(res, ContractError::WhitelistFrozen {});

        // Try adding members after freeze
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(100_000);
        let info = mock_info("creator", &[]);
        // Try adding members after whitelist ended
        let res = add_members(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            vec!["addr101".to_string()],
        )
        .unwrap_err();

        assert_eq!(res, ContractError::WhitelistFrozen {});

        // Try removing members after freeze
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(100_000);
        let info = mock_info("creator", &[]);
        // Try removing members after whitelist ended
        let res = remove_members(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            vec!["addr0".to_string()],
        )
        .unwrap_err();

        assert_eq!(res, ContractError::WhitelistFrozen {});

        // Try updating per address limit after freeze
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(100_000);
        let info = mock_info("creator", &[]);
        // Try updating per address limit after whitelist ended
        let res = update_per_address_limit(deps.as_mut(), env.clone(), info.clone(), 2, None)
            .unwrap_err();
        assert_eq!(res, ContractError::WhitelistFrozen {});

        // Try increasing member limit after freeze
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(100_000);
        let info = mock_info("creator", &[]);
        // Try increasing member limit after whitelist ended
        let res = increase_member_limit(deps.as_mut(), env.clone(), info.clone(), 2).unwrap_err();
        assert_eq!(res, ContractError::WhitelistFrozen {});

        // Try updating admin after freeze
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(100_000);
        let info = mock_info("creator", &[]);
        // Try updating admin after whitelist ended
        let res = update_admin(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            "addr101".to_string(),
        )
        .unwrap_err();
        assert_eq!(res, ContractError::WhitelistFrozen {});

        // Try freezing after freeze
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(100_000);
        let info = mock_info("creator", &[]);
        let res = freeze(deps.as_mut(), env.clone(), info.clone()).unwrap_err();
        assert_eq!(res, ContractError::WhitelistFrozen {});
    }
}
