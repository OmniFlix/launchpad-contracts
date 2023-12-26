use std::str::FromStr;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Params, PARAMS};
use crate::utils::check_payment;
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, to_json_binary, BankMsg, Binary, Coin, CosmosMsg, Decimal, Deps, DepsMut, Env,
    MessageInfo, Order, Response, StdResult, Timestamp, Uint128, WasmMsg,
};
use cw_utils::{may_pay, maybe_addr, must_pay, nonpayable};
use minter_types::InstantiateMsg as MinterInstantiateMsg;
use omniflix_std::types::omniflix::onft::v1beta1::OnftQuerier;
#[cfg(not(test))]
const CREATION_FEE: Uint128 = Uint128::new(0);
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
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let admin = maybe_addr(deps.api, msg.admin)?.unwrap_or(info.sender);
    let fee_collector_address = deps.api.addr_validate(&msg.fee_collector_address)?;
    if msg.minter_code_id == 0 {
        return Err(ContractError::InvalidMinterCodeId {});
    }
    let params = Params {
        admin: admin.clone(),
        allowed_minter_mint_denoms: msg.allowed_minter_mint_denoms,
        fee_collector_address: fee_collector_address,
        minter_code_id: msg.minter_code_id,
        minter_creation_fee: msg.minter_creation_fee,
    };
    PARAMS.save(deps.storage, &params)?;
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
        ExecuteMsg::CreateMinter { msg } => create_minter(deps, env, info, msg),
        ExecuteMsg::UpdateParams {
            admin,
            allowed_mint_denoms,
            fee_collector_address,
            minter_code_id,
            minter_creation_fee,
        } => update_params(
            deps,
            env,
            info,
            admin,
            allowed_mint_denoms,
            fee_collector_address,
            minter_code_id,
            minter_creation_fee,
        ),
    }
}

fn create_minter(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: MinterInstantiateMsg,
) -> Result<Response, ContractError> {
    let params = PARAMS.load(deps.storage)?;
    let nft_creation_fee: Coin = if CREATION_FEE == Uint128::new(0) {
        let onft_querier = OnftQuerier::new(&deps.querier);
        let params = onft_querier.params()?;
        let denom_creation_fee = params.params.unwrap().denom_creation_fee.unwrap();
        Coin {
            amount: Uint128::from_str(&denom_creation_fee.amount)?,
            denom: denom_creation_fee.denom,
        }
    } else {
        Coin {
            amount: CREATION_FEE,
            denom: CREATION_FEE_DENOM.to_string(),
        }
    };
    check_payment(
        &info.funds,
        &[nft_creation_fee.clone(), params.minter_creation_fee.clone()],
    )?;

    if !params.allowed_minter_mint_denoms.contains(&msg.mint_denom) {
        return Err(ContractError::MintDenomNotAllowed {});
    }

    let msgs: Vec<CosmosMsg> = vec![
        CosmosMsg::Wasm(WasmMsg::Instantiate {
            admin: Some(params.admin.to_string()),
            code_id: params.minter_code_id,
            msg: to_json_binary(&msg)?,
            funds: vec![nft_creation_fee],
            label: "omniflix-nft-minter".to_string(),
        }),
        CosmosMsg::Bank(BankMsg::Send {
            amount: vec![params.minter_creation_fee],
            to_address: params.fee_collector_address.to_string(),
        }),
    ];
    let res = Response::new()
        .add_messages(msgs)
        .add_attribute("action", "create_minter");
    Ok(res)
}

fn update_params(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    admin: Option<String>,
    allowed_mint_denoms: Option<Vec<String>>,
    fee_collector_address: Option<String>,
    minter_code_id: Option<u64>,
    minter_creation_fee: Option<Coin>,
) -> Result<Response, ContractError> {
    let mut params = PARAMS.load(deps.storage)?;
    if info.sender != params.admin {
        return Err(ContractError::Unauthorized {});
    }
    if let Some(admin) = admin {
        params.admin = deps.api.addr_validate(&admin)?;
    }
    if let Some(allowed_mint_denoms) = allowed_mint_denoms {
        params.allowed_minter_mint_denoms = allowed_mint_denoms;
    }
    if let Some(fee_collector_address) = fee_collector_address {
        params.fee_collector_address = deps.api.addr_validate(&fee_collector_address)?;
    }
    if let Some(minter_code_id) = minter_code_id {
        params.minter_code_id = minter_code_id;
    }
    if let Some(minter_creation_fee) = minter_creation_fee {
        params.minter_creation_fee = minter_creation_fee;
    }
    PARAMS.save(deps.storage, &params)?;
    Ok(Response::new().add_attribute("action", "update_params"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Params {} => to_json_binary(&query_params(deps)?),
    }
}

fn query_params(deps: Deps) -> StdResult<Params> {
    let params = PARAMS.load(deps.storage)?;
    Ok(params)
}
