#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, to_json_binary, wasm_execute, Binary, Coin, Deps, DepsMut, Env, MessageInfo, Order,
    Response, StdResult, Timestamp, Uint128, WasmMsg,
};
use cw2::set_contract_version;
use cw_utils::{maybe_addr, must_pay};

use crate::error::ContractError;
use crate::msg::ExecuteMsg;
use crate::state::{Config, RoundMethods, RoundMints, Rounds, CONFIG, ROUNDS_KEY, ROUND_MINTS};
use crate::utils::check_round_overlaps;
use whitelist_types::{
    InstantiateMsg, IsActiveResponse, IsMemberResponse, MembersResponse, MintPriceResponse, Round,
    RoundWhitelistQueryMsgs,
};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, "whitelist-round", "1.0.0");

    // TODO: Check instantiator is factory contract
    // After factory is written import params of the contract
    // Try parsing the response to the params struct

    let admin = maybe_addr(deps.api, msg.admin)?.unwrap_or(info.sender);
    // // Put index from 1 to n for rounds and return the rounds as Vec(index, round)
    // let rounds: Vec<(usize, Round)> = msg.rounds.into_iter().enumerate().collect::<Vec<_>>();
    let rounds = msg.rounds;
    // Check if rounds are valid
    for round in rounds.clone() {
        round.check_integrity(deps.as_ref(), env.block.time)?;
    }
    // Check if rounds overlap
    check_round_overlaps(rounds.clone())?;

    // Save rounds
    for round in rounds {
        Rounds::new(ROUNDS_KEY).save(deps.storage, &round)?;
    }

    let config = Config {
        admin: admin.clone(),
    };
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::default())
}
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::RemoveRound { round_index } => {
            execute_remove_round(deps, env, info, round_index)
        }
        ExecuteMsg::AddRound { round } => execute_add_round(deps, env, info, round),
        ExecuteMsg::PrivateMint { minter, admin } => {
            execute_private_mint(deps, env, info, minter, admin)
        }
    }
}
pub fn execute_remove_round(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    round_index: u32,
) -> Result<Response, ContractError> {
    // Check if sender is admin
    let config = CONFIG.load(deps.storage)?;
    let rounds = Rounds::new(ROUNDS_KEY);
    if info.sender != config.admin {
        return Err(ContractError::Unauthorized {});
    }
    // Check if the round exists
    let round = rounds.load(deps.storage, round_index)?;
    // Check if the round has started
    if round.start_time() < env.block.time {
        return Err(ContractError::RoundAlreadyStarted {});
    }
    // Remove the round
    rounds.remove(deps.storage, round_index)?;

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
    if info.sender != config.admin {
        return Err(ContractError::Unauthorized {});
    }
    round.check_integrity(deps.as_ref(), env.block.time)?;
    let rounds = Rounds::new(ROUNDS_KEY);
    // Check overlaps
    rounds.check_round_overlaps(deps.storage, Some(round.clone()))?;
    // Save the round
    let new_round_index = rounds.save(deps.storage, &round)?;

    let res = Response::new()
        .add_attribute("action", "add_round")
        .add_attribute("round_index", (new_round_index).to_string());

    Ok(res)
}

pub fn execute_private_mint(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    minter: String,
    admin: String,
) -> Result<Response, ContractError> {
    // Load config
    let config = CONFIG.load(deps.storage)?;
    // Check if the msg admin is same as the config admin
    let admin = deps.api.addr_validate(admin.as_str())?;
    let minter = deps.api.addr_validate(minter.as_str())?;

    if config.admin != admin {
        return Err(ContractError::Unauthorized {});
    };
    // TODO: Query if sender is a minter contract
    let rounds = Rounds::new(ROUNDS_KEY);

    // Find active round
    let active_round = rounds.load_active_round(deps.storage, env.block.time);
    if active_round.is_none() {
        return Err(ContractError::NoActiveRound {});
    };
    let active_round = active_round.unwrap();
    // Load round mints for the address
    let mut round_mints = ROUND_MINTS
        .load(deps.storage, minter.clone())
        .unwrap_or(RoundMints::new());
    // Check if address has already reached the limit for the round. If not reached mint count will be incremented
    round_mints.try_mint(active_round.clone())?;
    // Save the round mints
    ROUND_MINTS.save(deps.storage, minter.clone(), &round_mints)?;

    let res = Response::new()
        .add_attribute("action", "privately_mint")
        .add_attribute("minter", minter.to_string());
    Ok(res)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: RoundWhitelistQueryMsgs) -> StdResult<Binary> {
    match msg {
        RoundWhitelistQueryMsgs::ActiveRound {} => to_json_binary(&query_active_round(deps, env)?),
        RoundWhitelistQueryMsgs::IsActive {} => to_json_binary(&query_is_active(deps, env)?),
        RoundWhitelistQueryMsgs::Members {
            round_index,
            start_after,
            limit,
        } => to_json_binary(&query_members(deps, env, round_index, start_after, limit)?),
        RoundWhitelistQueryMsgs::Price {} => to_json_binary(&query_price(deps, env)?),
        RoundWhitelistQueryMsgs::Rounds {} => to_json_binary(&query_rounds(deps, env)?),
        RoundWhitelistQueryMsgs::Round { round_index } => {
            to_json_binary(&query_round(deps, round_index)?)
        }
        RoundWhitelistQueryMsgs::IsMember { address } => {
            to_json_binary(&query_is_member(deps, env, address)?)
        }
    }
}

pub fn query_active_round(deps: Deps, env: Env) -> Result<Round, ContractError> {
    let rounds = Rounds::new(ROUNDS_KEY);
    let active_round = rounds.load_active_round(deps.storage, env.block.time);
    let active_round = match active_round {
        Some(active_round) => active_round,
        None => return Err(ContractError::NoActiveRound {}),
    };
    Ok(active_round)
}

pub fn query_is_active(deps: Deps, env: Env) -> Result<IsActiveResponse, ContractError> {
    let rounds = Rounds::new(ROUNDS_KEY);
    let active_round = rounds.load_active_round(deps.storage, env.block.time);
    let is_active = match active_round {
        Some(_) => true,
        None => false,
    };
    Ok(IsActiveResponse {
        is_active: is_active,
    })
}

pub fn query_members(
    deps: Deps,
    env: Env,
    round_index: u32,
    start_after: Option<String>,
    limit: Option<u32>,
) -> Result<MembersResponse, ContractError> {
    let rounds = Rounds::new(ROUNDS_KEY);
    let round = rounds.load(deps.storage, round_index)?;
    let members = round.members(start_after, limit)?;
    let res = MembersResponse { members };
    Ok(res)
}

pub fn query_price(deps: Deps, env: Env) -> Result<MintPriceResponse, ContractError> {
    let rounds = Rounds::new(ROUNDS_KEY);
    let active_round = rounds.load_active_round(deps.storage, env.block.time);
    let active_round = match active_round {
        Some(active_round) => active_round,
        None => return Err(ContractError::NoActiveRound {}),
    };
    let price = active_round.mint_price();
    Ok(MintPriceResponse { mint_price: price })
}

pub fn query_rounds(deps: Deps, env: Env) -> Result<Vec<Round>, ContractError> {
    let rounds = Rounds::new(ROUNDS_KEY);
    let rounds = rounds.load_all_rounds(deps.storage)?;
    Ok(rounds)
}

pub fn query_round(deps: Deps, round_index: u32) -> Result<Round, ContractError> {
    let rounds = Rounds::new(ROUNDS_KEY);
    let round = rounds.load(deps.storage, round_index)?;
    Ok(round)
}

pub fn query_is_member(
    deps: Deps,
    env: Env,
    address: String,
) -> Result<IsMemberResponse, ContractError> {
    let rounds = Rounds::new(ROUNDS_KEY);
    let active_round = rounds.load_active_round(deps.storage, env.block.time);
    let active_round = match active_round {
        Some(active_round) => active_round,
        None => return Err(ContractError::NoActiveRound {}),
    };
    let address = deps.api.addr_validate(&address)?;
    let is_member = active_round.is_member(&address);
    Ok(IsMemberResponse {
        is_member: is_member,
    })
}
