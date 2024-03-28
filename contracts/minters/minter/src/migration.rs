use std::env;
use std::str::FromStr;

use crate::msg::{ExecuteMsg, MinterExtensionQueryMsg};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Addr, Binary, Coin, CosmosMsg, Decimal, Deps, DepsMut, Env, MessageInfo, Order,
    Response, StdResult, Uint128, WasmMsg,
};
use cw_utils::{may_pay, maybe_addr, must_pay, nonpayable};
use minter_types::utils::{
    check_collection_creation_fee, generate_create_denom_msg, generate_minter_mint_message,
    generate_update_denom_msg, update_collection_details,
};
use omniflix_minter_factory::msg::QueryMsg::Params as QueryFactoryParams;
use omniflix_minter_factory::msg::{CreateMinterMsg, ParamsResponse};
use omniflix_round_whitelist::msg::ExecuteMsg::PrivateMint;
use whitelist_types::{
    check_if_address_is_member, check_if_whitelist_is_active, check_whitelist_price,
};

use crate::error::ContractError;
use crate::state::{
    AUTH_DETAILS, COLLECTION, CONFIG, MINTABLE_TOKENS, TOKEN_DETAILS, TOTAL_TOKENS_REMAINING,
    USER_MINTING_DETAILS,
};
use crate::utils::{
    collect_mintable_tokens, generate_tokens, randomize_token_list, return_random_token,
};
use minter_types::msg::QueryMsg as BaseMinterQueryMsg;
use minter_types::types::{
    AuthDetails, CollectionDetails, Config, Token, TokenDetails, UserDetails,
};
use pauser::PauseState;

use cw2::set_contract_version;
use omniflix_std::types::omniflix::onft::v1beta1::{MsgPurgeDenom, WeightedAddress};
// TODO RENAME THIS
pub fn instantiate_with_migration(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: CreateMinterMsg,
) -> Result<Response, ContractError> {
    // We check if the migration data is present
    if msg.migration_data.is_none() {
        return Err(ContractError::MigrationDataNotFound {});
    }

    // Query factory params of instantiator
    let _factory_params: ParamsResponse = deps
        .querier
        .query_wasm_smart(info.sender.clone().into_string(), &QueryFactoryParams {})?;

    // Get the migration data
    let migration_data = msg.migration_data.unwrap();
    let init = msg.init.clone();
    let collection_details = msg.collection_details.clone();
    collection_details.check_integrity()?;

    let token_details = TokenDetails::default();
    let config = Config {
        start_time: init.start_time,
        end_time: init.end_time,
        per_address_limit: init.per_address_limit,
        mint_price: init.mint_price,
        whitelist_address: maybe_addr(deps.api, init.whitelist_address)?,
        num_tokens: Some(init.num_tokens),
    };

    Ok(Response::default())
}
