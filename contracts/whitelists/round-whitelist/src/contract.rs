#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, to_json_binary, wasm_execute, Binary, Coin, Deps, DepsMut, Env, MessageInfo, Order,
    Response, StdResult, Timestamp, Uint128, WasmMsg,
};
use cw2::set_contract_version;
use cw_utils::{maybe_addr, must_pay};

use types::whitelist::{
    Config, HasEndedResponse, HasMemberResponse, HasStartedResponse, IsActiveResponse,
    MembersResponse, PerAddressLimitResponse, WhitelistQueryMsgs,
};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg};

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

    Ok(Response::default())
}

// // If not public mint try to find the rounds
// let rounds: StdResult<Vec<(u32, Round)>> = ROUNDS
//     .range(deps.storage, None, None, Order::Ascending)
//     .collect();
// let rounds = rounds.unwrap_or(Vec::new());
// if rounds.is_empty() {
//     return Err(ContractError::MintingNotStarted {
//         start_time: config.start_time.nanos(),
//         current_time: env.block.time.nanos(),
//     });
// }
// // Check if any active round exists
// let active_round = find_active_round(env.block.time, rounds)?;
// let active_round_index = active_round.0;
// // Check if the address is whitelisted
// let is_member = check_if_whitelisted(
//     info.sender.clone().into_string(),
//     active_round.1.clone(),
//     deps.as_ref(),
// )?;
// if !is_member {
//     return Err(ContractError::AddressNotWhitelisted {});
// }
// // This function tries to add mintable token to user details
// // If succesfully updates it that means per_address_limit or round limit is not reached
// user_details.add_minted_token(
//     config.per_address_limit,
//     Some(active_round.1.round_limit()),
//     random_token.clone().1,
//     Some(active_round_index),
// )?;
// // Determine mint price
// mint_price = active_round.1.mint_price()

// pub fn execute_remove_round(
//     deps: DepsMut,
//     env: Env,
//     info: MessageInfo,
//     round_index: u32,
// ) -> Result<Response, ContractError> {
//     // Check if sender is admin
//     let config = CONFIG.load(deps.storage)?;
//     if info.sender != config.creator {
//         return Err(ContractError::Unauthorized {});
//     }
//     // Check if the round exists
//     let round = ROUNDS.may_load(deps.storage, round_index)?;
//     if round.is_none() {
//         return Err(ContractError::RoundNotFound {});
//     }
//     // Check if the round has started
//     let round = round.unwrap();
//     if round.start_time() < env.block.time {
//         return Err(ContractError::RoundAlreadyStarted {});
//     }
//     // Remove the round
//     ROUNDS.remove(deps.storage, round_index);

//     let res = Response::new()
//         .add_attribute("action", "remove_round")
//         .add_attribute("round_index", round_index.to_string());
//     Ok(res)
// }

// pub fn execute_add_round(
//     deps: DepsMut,
//     env: Env,
//     info: MessageInfo,
//     round: Round,
// ) -> Result<Response, ContractError> {
//     // Check if sender is admin
//     let config = CONFIG.load(deps.storage)?;
//     if info.sender != config.creator {
//         return Err(ContractError::Unauthorized {});
//     }
//     // Check round type
//     let rounds = ROUNDS
//         .range(deps.storage, None, None, Order::Ascending)
//         .collect::<StdResult<Vec<(u32, Round)>>>()?;
//     let round_exists = check_if_round_exists(&round, rounds.clone());
//     let latest_index = rounds.iter().map(|(i, _)| i).max().unwrap_or(&0);

//     if round_exists {
//         return Err(ContractError::RoundAlreadyExists {});
//     }
//     let updated_round = return_updated_round(&deps, round.clone())?;
//     // Check if the round start time is valid
//     if updated_round.start_time() < env.block.time {
//         return Err(ContractError::RoundAlreadyStarted {});
//     }
//     let mut updated_rounds = rounds.clone();
//     updated_rounds.push((*latest_index as u32 + 1, updated_round.clone()));
//     // Check if rounds overlap
//     check_round_overlaps(env.block.time, updated_rounds, config.start_time)?;
//     ROUNDS.save(deps.storage, *latest_index as u32 + 1, &updated_round)?;

//     let res = Response::new()
//         .add_attribute("action", "add_round")
//         .add_attribute("round_index", (*latest_index as u32 + 1).to_string());

//     Ok(res)
// }

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

// pub fn check_round_overlaps(
//     now: Timestamp,
//     rounds: Vec<(u32, Round)>,
//     public_start_time: Timestamp,
// ) -> Result<(), ContractError> {
//     let mut rounds = rounds;

//     // add public as a round
//     rounds.push((
//         u32::MAX,
//         Round::WhitelistAddress {
//             address: Addr::unchecked("public"),
//             start_time: Some(public_start_time),
//             // There is no public mint end time we generate 10_000 day after start time to be safe
//             // Only to check for overlaps
//             end_time: Some(public_start_time.plus_days(10_000)),
//             mint_price: Default::default(),
//             round_limit: Default::default(),
//         },
//     ));
//     // Sort rounds by start time
//     rounds.sort_by(|a, b| a.1.start_time().cmp(&b.1.start_time()));
//     // Check for overlaps
//     for (i, round) in rounds.iter().enumerate() {
//         if i == rounds.len() - 1 {
//             break;
//         }
//         // Check for start time can not be bigger than end time
//         if round.1.start_time() > round.1.end_time() {
//             return Err(ContractError::InvalidRoundTime {
//                 round: round.1.clone(),
//             });
//         }
//         let next_round = &rounds[i + 1];
//         if round.1.end_time() > next_round.1.start_time() {
//             return Err(ContractError::RoundsOverlaped {
//                 round: round.1.clone(),
//             });
//         }
//     }
//     // Check for overlaps with now none of them should be started
//     for round in rounds {
//         if round.1.start_time() < now {
//             return Err(ContractError::RoundAlreadyStarted {});
//         }
//     }
//     Ok(())
// }
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
