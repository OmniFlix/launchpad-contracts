use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, ParamsResponse, QueryMsg};
use crate::state::{Params, PARAMS};
use crate::utils::check_payment;
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, BankMsg, Binary, Coin, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response,
    StdResult, Uint128, WasmMsg,
};
use cw_utils::maybe_addr;
use omniflix_std::types::omniflix::onft::v1beta1::OnftQuerier;
use open_edition_minter_types::InstantiateMsg as OpenEditionMinterInstantiateMsg;
use std::str::FromStr;
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
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let admin = maybe_addr(deps.api, msg.admin)?.unwrap_or(info.sender);
    let fee_collector_address = deps.api.addr_validate(&msg.fee_collector_address)?;
    if msg.open_edition_minter_code_id == 0 {
        return Err(ContractError::InvalidMinterCodeId {});
    }
    let params = Params {
        admin: admin.clone(),
        allowed_minter_mint_denoms: msg.allowed_minter_mint_denoms,
        fee_collector_address,
        open_edition_minter_code_id: msg.open_edition_minter_code_id,
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
        ExecuteMsg::UpdateAdmin { admin } => update_params_admin(deps, env, info, admin),
        ExecuteMsg::UpdateAllowedMinterMintDenoms {
            allowed_minter_mint_denoms,
        } => update_params_allowed_mint_denoms(deps, env, info, allowed_minter_mint_denoms),
        ExecuteMsg::UpdateFeeCollectorAddress {
            fee_collector_address,
        } => update_params_fee_collector_address(deps, env, info, fee_collector_address),
        ExecuteMsg::UpdateMinterCodeId { minter_code_id } => {
            update_params_minter_code_id(deps, env, info, minter_code_id)
        }
        ExecuteMsg::UpdateMinterCreationFee {
            minter_creation_fee,
        } => update_params_minter_creation_fee(deps, env, info, minter_creation_fee),
    }
}

fn create_minter(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: OpenEditionMinterInstantiateMsg,
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
            code_id: params.open_edition_minter_code_id,
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

fn update_params_admin(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    admin: String,
) -> Result<Response, ContractError> {
    let mut params = PARAMS.load(deps.storage)?;
    if params.admin != info.sender {
        return Err(ContractError::Unauthorized {});
    }
    params.admin = deps.api.addr_validate(&admin)?;
    PARAMS.save(deps.storage, &params)?;
    Ok(Response::default()
        .add_attribute("action", "update_admin")
        .add_attribute("new_admin", admin))
}

fn update_params_allowed_mint_denoms(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    allowed_mint_denoms: Vec<String>,
) -> Result<Response, ContractError> {
    let mut params = PARAMS.load(deps.storage)?;
    if params.admin != info.sender {
        return Err(ContractError::Unauthorized {});
    }
    params.allowed_minter_mint_denoms = allowed_mint_denoms.clone();
    PARAMS.save(deps.storage, &params)?;
    Ok(Response::default()
        .add_attribute("action", "update_allowed_mint_denoms")
        .add_attribute("new_allowed_mint_denoms", allowed_mint_denoms.join(",")))
}

fn update_params_fee_collector_address(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    fee_collector_address: String,
) -> Result<Response, ContractError> {
    let mut params = PARAMS.load(deps.storage)?;
    if params.admin != info.sender {
        return Err(ContractError::Unauthorized {});
    }
    params.fee_collector_address = deps.api.addr_validate(&fee_collector_address)?;
    PARAMS.save(deps.storage, &params)?;
    Ok(Response::default()
        .add_attribute("action", "update_fee_collector_address")
        .add_attribute(
            "new_fee_collector_address",
            fee_collector_address.to_string(),
        ))
}

fn update_params_minter_code_id(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    minter_code_id: u64,
) -> Result<Response, ContractError> {
    let mut params = PARAMS.load(deps.storage)?;
    if params.admin != info.sender {
        return Err(ContractError::Unauthorized {});
    }
    params.open_edition_minter_code_id = minter_code_id;
    PARAMS.save(deps.storage, &params)?;
    Ok(Response::default()
        .add_attribute("action", "update_minter_code_id")
        .add_attribute("new_minter_code_id", minter_code_id.to_string()))
}

fn update_params_minter_creation_fee(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    minter_creation_fee: Coin,
) -> Result<Response, ContractError> {
    let mut params = PARAMS.load(deps.storage)?;
    if params.admin != info.sender {
        return Err(ContractError::Unauthorized {});
    }
    params.minter_creation_fee = minter_creation_fee.clone();
    PARAMS.save(deps.storage, &params)?;
    Ok(Response::default()
        .add_attribute("action", "update_minter_creation_fee")
        .add_attribute("new_minter_creation_fee", minter_creation_fee.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Params {} => to_json_binary(&query_params(deps)?),
    }
}

fn query_params(deps: Deps) -> StdResult<ParamsResponse> {
    let params = PARAMS.load(deps.storage)?;
    Ok(ParamsResponse { params })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::{
        testing::{mock_dependencies, mock_env, mock_info},
        Addr, Decimal, Timestamp,
    };
    use open_edition_minter_types::CollectionDetails;

    #[test]
    fn test_instantiate() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg {
            admin: None,
            allowed_minter_mint_denoms: vec!["uusd".to_string(), "uflix".to_string()],
            fee_collector_address: "fee_collector_address".to_string(),
            open_edition_minter_code_id: 1,
            minter_creation_fee: Coin {
                amount: Uint128::new(100),
                denom: "uusd".to_string(),
            },
        };
        let info = mock_info("creator", &[]);
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // query params
        let params = query_params(deps.as_ref()).unwrap();
        assert_eq!(
            params.params,
            Params {
                admin: Addr::unchecked("creator"),
                allowed_minter_mint_denoms: vec!["uusd".to_string(), "uflix".to_string()],
                fee_collector_address: Addr::unchecked("fee_collector_address"),
                open_edition_minter_code_id: 1,
                minter_creation_fee: Coin {
                    amount: Uint128::new(100),
                    denom: "uusd".to_string(),
                },
            }
        );
    }

    #[test]
    fn test_execute_create_open_edition_minter() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg {
            admin: None,
            allowed_minter_mint_denoms: vec!["uusd".to_string(), "uflix".to_string()],
            fee_collector_address: "fee_collector_address".to_string(),
            open_edition_minter_code_id: 1,
            minter_creation_fee: Coin {
                amount: Uint128::new(100),
                denom: "uusd".to_string(),
            },
        };
        let info = mock_info("creator", &[]);
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        let collection_details = CollectionDetails {
            name: "My Collection".to_string(),
            description: "This is a collection of unique tokens.".to_string(),
            preview_uri: "https://example.com/preview".to_string(),
            schema: "https://example.com/schema".to_string(),
            symbol: "SYM".to_string(),
            id: "collection_id".to_string(),
            extensible: true,
            nsfw: false,
            base_uri: "https://example.com/base".to_string(),
            uri: "https://example.com/collection".to_string(),
            uri_hash: "hash123".to_string(),
            data: "Additional data for the collection".to_string(),
        };
        // Non allowed mint denom
        let msg = ExecuteMsg::CreateMinter {
            msg: OpenEditionMinterInstantiateMsg {
                admin: None,
                whitelist_address: None,
                mint_denom: "non_allowed".to_string(),
                mint_price: Uint128::new(100),
                start_time: Timestamp::from_seconds(0),
                royalty_ratio: Decimal::percent(10).to_string(),
                payment_collector: None,
                per_address_limit: 3,
                collection_details: collection_details.clone(),
                end_time: None,
                token_limit: None,
            },
        };

        let info = mock_info(
            "creator",
            &[
                Coin {
                    amount: Uint128::new(100_000_000),
                    denom: "uflix".to_string(),
                },
                Coin {
                    amount: Uint128::new(100),
                    denom: "uusd".to_string(),
                },
            ],
        );
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
        assert_eq!(res, ContractError::MintDenomNotAllowed {});
        // Send additional funds
        let msg = ExecuteMsg::CreateMinter {
            msg: OpenEditionMinterInstantiateMsg {
                admin: None,
                whitelist_address: None,
                mint_denom: "uusd".to_string(),
                mint_price: Uint128::new(100),
                start_time: Timestamp::from_seconds(0),
                royalty_ratio: Decimal::percent(10).to_string(),
                payment_collector: None,
                per_address_limit: 3,
                collection_details: collection_details.clone(),
                end_time: None,
                token_limit: None,
            },
        };

        let info = mock_info(
            "creator",
            &[
                Coin {
                    amount: Uint128::new(100_000_000),
                    denom: "uflix".to_string(),
                },
                Coin {
                    amount: Uint128::new(100),
                    denom: "uusd".to_string(),
                },
                Coin {
                    amount: Uint128::new(100),
                    denom: "additional".to_string(),
                },
            ],
        );
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
        assert_eq!(
            res,
            ContractError::IncorrectFunds {
                expected: vec![
                    Coin {
                        amount: Uint128::new(100_000_000),
                        denom: "uflix".to_string(),
                    },
                    Coin {
                        amount: Uint128::new(100),
                        denom: "uusd".to_string(),
                    },
                ],
                actual: vec![
                    Coin {
                        amount: Uint128::new(100_000_000),
                        denom: "uflix".to_string(),
                    },
                    Coin {
                        amount: Uint128::new(100),
                        denom: "uusd".to_string(),
                    },
                    Coin {
                        amount: Uint128::new(100),
                        denom: "additional".to_string(),
                    },
                ],
            }
        );

        // Missing funds
        let msg = ExecuteMsg::CreateMinter {
            msg: OpenEditionMinterInstantiateMsg {
                admin: None,
                whitelist_address: None,
                mint_denom: "uusd".to_string(),
                mint_price: Uint128::new(100),
                start_time: Timestamp::from_seconds(0),
                royalty_ratio: Decimal::percent(10).to_string(),
                payment_collector: None,
                per_address_limit: 3,
                collection_details: collection_details.clone(),
                end_time: None,
                token_limit: None,
            },
        };

        let info = mock_info(
            "creator",
            &[Coin {
                amount: Uint128::new(100_000_000),
                denom: "uflix".to_string(),
            }],
        );
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
        assert_eq!(
            res,
            ContractError::IncorrectFunds {
                expected: vec![
                    Coin {
                        amount: Uint128::new(100_000_000),
                        denom: "uflix".to_string(),
                    },
                    Coin {
                        amount: Uint128::new(100),
                        denom: "uusd".to_string(),
                    },
                ],
                actual: vec![Coin {
                    amount: Uint128::new(100_000_000),
                    denom: "uflix".to_string(),
                },],
            }
        );

        // Happy path
        let msg = ExecuteMsg::CreateMinter {
            msg: OpenEditionMinterInstantiateMsg {
                admin: None,
                whitelist_address: None,
                mint_denom: "uusd".to_string(),
                mint_price: Uint128::new(100),
                start_time: Timestamp::from_seconds(0),
                royalty_ratio: Decimal::percent(10).to_string(),
                payment_collector: None,
                per_address_limit: 3,
                collection_details: collection_details.clone(),
                end_time: None,
                token_limit: None,
            },
        };

        let info = mock_info(
            "creator",
            &[
                Coin {
                    amount: Uint128::new(100_000_000),
                    denom: "uflix".to_string(),
                },
                Coin {
                    amount: Uint128::new(100),
                    denom: "uusd".to_string(),
                },
            ],
        );
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        assert_eq!(res.messages.len(), 2);
        assert_eq!(
            res.messages[0].msg,
            CosmosMsg::Wasm(WasmMsg::Instantiate {
                admin: Some("creator".to_string()),
                code_id: 1,
                msg: to_json_binary(&OpenEditionMinterInstantiateMsg {
                    admin: None,
                    whitelist_address: None,
                    mint_denom: "uusd".to_string(),
                    mint_price: Uint128::new(100),
                    start_time: Timestamp::from_seconds(0),
                    royalty_ratio: Decimal::percent(10).to_string(),
                    payment_collector: None,
                    per_address_limit: 3,
                    collection_details: collection_details.clone(),
                    end_time: None,
                    token_limit: None,
                })
                .unwrap(),
                funds: vec![Coin {
                    amount: Uint128::new(100_000_000),
                    denom: "uflix".to_string(),
                }],
                label: "omniflix-nft-minter".to_string(),
            })
        );
        assert_eq!(
            res.messages[1].msg,
            CosmosMsg::Bank(BankMsg::Send {
                amount: vec![Coin {
                    amount: Uint128::new(100),
                    denom: "uusd".to_string(),
                }],
                to_address: "fee_collector_address".to_string(),
            })
        );
    }
}
