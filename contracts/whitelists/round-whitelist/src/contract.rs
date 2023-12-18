#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, to_json_binary, wasm_execute, Binary, Coin, Deps, DepsMut, Env, MessageInfo, Order,
    Response, StdResult, Timestamp, Uint128, WasmMsg,
};
use cw2::set_contract_version;
use cw_utils::{maybe_addr, must_pay};

use types::whitelist::{
    HasEndedResponse, HasMemberResponse, HasStartedResponse, IsActiveResponse, MembersResponse,
    PerAddressLimitResponse, WhitelistQueryMsgs,
};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg};
use crate::state::{Config, Round, RoundMints, Rounds, CONFIG, ROUNDS_KEY, ROUND_MINTS};
use crate::utils::check_round_overlaps;

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
    check_round_overlaps(env.block.time, rounds.clone())?;

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
        ExecuteMsg::PrivatelyMint { minter, admin } => {
            execute_privately_mint(deps, env, info, minter)
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
    rounds.check_round_overlaps(deps.storage, Some(round))?;
    // Save the round
    let new_round_index = rounds.save(deps.storage, &round)?;

    let res = Response::new()
        .add_attribute("action", "add_round")
        .add_attribute("round_index", (new_round_index).to_string());

    Ok(res)
}

pub fn execute_privately_mint(
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
        .load(deps.storage, minter)
        .unwrap_or(RoundMints { rounds: vec![] });
}

// pub fn execute_update_collection_round(
//     deps: DepsMut,
//     _env: Env,
//     info: MessageInfo,
//     round_index: u32,
//     round: Round,
// ) -> Result<Response, ContractError> {
//     // Check if sender is admin
//     let config = CONFIG.load(deps.storage)?;
//     if info.sender != config.creator {
//         return Err(ContractError::Unauthorized {});
//     }
//     let new_round = round.clone();
//     let new_round_type = new_round.return_round_type();
//     if new_round_type != "collection" {
//         return Err(ContractError::InvalidRoundType {
//             expected: "collection".to_string(),
//             actual: new_round_type,
//         });
//     }
//     // Find round with round_index
//     let older_round = ROUNDS.may_load(deps.storage, round_index)?;
//     if older_round.is_none() {
//         return Err(ContractError::RoundNotFound {});
//     }
//     let older_round = older_round.unwrap();
//     let round_type = older_round.return_round_type();
//     if round_type != "collection" {
//         return Err(ContractError::InvalidRoundType {
//             expected: "collection".to_string(),
//             actual: round_type,
//         });
//     }
//     // Load all the rounds
//     let all_rounds = ROUNDS
//         .range(deps.storage, None, None, Order::Ascending)
//         .collect::<StdResult<Vec<(u32, Round)>>>()?;
//     // Remove older round from the list
//     let mut new_rounds = all_rounds.clone();
//     new_rounds.retain(|(i, _)| i != &round_index);
//     // Add the new round to the list
//     new_rounds.push((round_index, new_round));

//     // Check if rounds overlap
//     check_round_overlaps(_env.block.time, new_rounds, config.start_time)?;

//     // If not overlapping remove older
//     ROUNDS.remove(deps.storage, round_index);
//     ROUNDS.save(deps.storage, round_index, &round)?;

//     let res = Response::new()
//         .add_attribute("action", "update_collection_round")
//         .add_attribute("round_index", round_index.to_string());

//     Ok(res)
// }

// pub fn execute_update_whitelist_round(
//     deps: DepsMut,
//     _env: Env,
//     info: MessageInfo,
//     start_time: Option<Timestamp>,
//     end_time: Option<Timestamp>,
//     mint_price: Option<Uint128>,
//     round_limit: Option<u32>,
// ) -> Result<Response, ContractError> {
//     // Check if sender is whitelist address
//     // We are expectiong this update to be called from the whitelist contract
//     let config = CONFIG.load(deps.storage)?;
//     let whitelist_address = info.sender.clone();
//     // Load all the rounds
//     let rounds = ROUNDS
//         .range(deps.storage, None, None, Order::Ascending)
//         .collect::<StdResult<Vec<(u32, Round)>>>()?;
//     // Find the round with whitelist address
//     let round = rounds
//         .iter()
//         .find(|(_, round)| match round {
//             Round::WhitelistAddress { address, .. } => address == &whitelist_address,
//             _ => false,
//         })
//         .ok_or(ContractError::RoundNotFound {})?;

//     // Update the round
//     let round_index = round.0;
//     let mut round = round.1.clone();
//     // Update the round
//     round.update_params(start_time, end_time, mint_price, round_limit)?;
//     let mut updated_rounds = rounds.clone();
//     // Remove the round from the list
//     updated_rounds.retain(|(i, _)| i != &round_index);
//     // Add the new round to the list
//     updated_rounds.push((round_index, round.clone()));

//     // Check if rounds overlap
//     check_round_overlaps(_env.block.time, updated_rounds, config.start_time)?;

//     // If not overlapping remove older from store
//     ROUNDS.remove(deps.storage, round_index);
//     ROUNDS.save(deps.storage, round_index, &round)?;

//     let res = Response::new()
//         .add_attribute("action", "update_whitelist_round")
//         .add_attribute("round_index", round_index.to_string());

//     Ok(res)
// }

//
// pub fn return_updated_round(deps: &DepsMut, round: Round) -> Result<Round, ContractError> {
//     match round {
//         Round::WhitelistAddress {
//             address,
//             start_time,
//             end_time,
//             mint_price,
//             round_limit,
//         } => {
//             let whitelist_config: WhitelistConfig = deps
//                 .querier
//                 .query_wasm_smart(address.clone(), &WhitelistQueryMsgs::Config {})?;
//             let round = Round::WhitelistAddress {
//                 address,
//                 start_time: Some(whitelist_config.start_time),
//                 end_time: Some(whitelist_config.end_time),
//                 mint_price: whitelist_config.mint_price.amount,
//                 round_limit: whitelist_config.per_address_limit,
//             };
//             Ok(round)
//         }
//         Round::WhitelistCollection {
//             collection_id,
//             start_time,
//             end_time,
//             mint_price,
//             round_limit,
//         } => {
//             let round = Round::WhitelistCollection {
//                 collection_id: collection_id.clone(),
//                 start_time,
//                 end_time,
//                 mint_price,
//                 round_limit,
//             };
//             Ok(round)
//         }
//     }
// }

// pub fn check_if_whitelisted(
//     member: String,
//     round: Round,
//     deps: Deps,
// ) -> Result<bool, ContractError> {
//     match round {
//         Round::WhitelistAddress {
//             address,
//             start_time,
//             end_time,
//             mint_price,
//             round_limit,
//         } => {
//             let has_member_response: HasMemberResponse = deps.querier.query_wasm_smart(
//                 address,
//                 &WhitelistQueryMsgs::HasMember {
//                     member: member.clone(),
//                 },
//             )?;
//             if has_member_response.has_member {
//                 return Ok(true);
//             }
//         }
//         Round::WhitelistCollection {
//             collection_id,
//             start_time,
//             end_time,
//             mint_price,
//             round_limit,
//         } => {
//             let onft_querier = OnftQuerier::new(&deps.querier);
//             // TODO: Check if there is better way
//             let owner_amount = onft_querier.supply(collection_id, member)?;
//             if owner_amount.amount > 0 {
//                 return Ok(true);
//             }
//         }
//     }

//     Ok(false)
// }

// pub fn find_active_round(
//     now: Timestamp,
//     rounds: Vec<(u32, Round)>,
// ) -> Result<(u32, Round), ContractError> {
//     let mut rounds = rounds;
//     // Sort rounds by start time
//     rounds.sort_by(|a, b| a.1.start_time().cmp(&b.1.start_time()));
//     // Find active round
//     for round in rounds {
//         if round.1.start_time() <= now && round.1.end_time() >= now {
//             return Ok(round);
//         }
//     }
//     Err(ContractError::RoundEnded {})
// }

// pub fn check_if_round_exists(round: &Round, rounds: Vec<(u32, Round)>) -> bool {
//     match round {
//         Round::WhitelistAddress { address, .. } => rounds.iter().any(|(_, r)| match r {
//             Round::WhitelistAddress {
//                 address: round_address,
//                 ..
//             } => address == round_address,
//             _ => false,
//         }),
//         Round::WhitelistCollection { collection_id, .. } => rounds.iter().any(|(_, r)| r == round),
//     }
// }

// impl Round {
//     //     pub fn start_time(&self) -> Timestamp {
//     //         match self {
//     //             Round::WhitelistAddress { start_time, .. } => start_time.unwrap(),
//     //             Round::WhitelistCollection { start_time, .. } => *start_time,
//     //         }
//     //     }
//     //     pub fn end_time(&self) -> Timestamp {
//     //         match self {
//     //             Round::WhitelistAddress { end_time, .. } => end_time.unwrap(),
//     //             Round::WhitelistCollection { end_time, .. } => *end_time,
//     //         }
//     //     }
//     //     pub fn mint_price(&self) -> Uint128 {
//     //         match self {
//     //             Round::WhitelistAddress { mint_price, .. } => *mint_price,
//     //             Round::WhitelistCollection { mint_price, .. } => *mint_price,
//     //         }
//     //     }
//     //     pub fn round_limit(&self) -> u32 {
//     //         match self {
//     //             Round::WhitelistAddress { round_limit, .. } => *round_limit,
//     //             Round::WhitelistCollection { round_limit, .. } => *round_limit,
//     //         }
//     //     }
//     //     pub fn return_whitelist_address(&self) -> Option<Addr> {
//     //         match self {
//     //             Round::WhitelistAddress { address, .. } => Some(address.clone()),
//     //             Round::WhitelistCollection { .. } => None,
//     //         }
//     //     }
//     pub fn update_params(
//         &mut self,
//         start_time: Option<Timestamp>,
//         end_time: Option<Timestamp>,
//         mint_price: Option<Uint128>,
//         round_limit: Option<u32>,
//     ) -> Result<(), ContractError> {
//         match self {
//             Round::WhitelistAddress {
//                 start_time: ref mut s,
//                 end_time: ref mut e,
//                 mint_price: ref mut m,
//                 round_limit: ref mut r,
//                 ..
//             } => {
//                 if let Some(start_time) = start_time {
//                     *s = Some(start_time);
//                 }
//                 if let Some(end_time) = end_time {
//                     *e = Some(end_time);
//                 }
//                 if let Some(mint_price) = mint_price {
//                     *m = mint_price;
//                 }
//                 if let Some(round_limit) = round_limit {
//                     *r = round_limit;
//                 }
//             }
//             Round::WhitelistCollection {
//                 start_time: ref mut s,
//                 end_time: ref mut e,
//                 mint_price: ref mut m,
//                 round_limit: ref mut r,
//                 ..
//             } => {
//                 if let Some(start_time) = start_time {
//                     *s = start_time;
//                 }
//                 if let Some(end_time) = end_time {
//                     *e = end_time;
//                 }
//                 if let Some(mint_price) = mint_price {
//                     *m = mint_price;
//                 }
//                 if let Some(round_limit) = round_limit {
//                     *r = round_limit;
//                 }
//             }
//         }
//         Ok(())
//     }
//     pub fn return_round_type(&self) -> String {
//         match self {
//             Round::WhitelistAddress { .. } => "address".to_string(),
//             Round::WhitelistCollection { .. } => "collection".to_string(),
//         }
//     }
// }
