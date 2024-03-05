#![cfg(test)]
use cosmwasm_std::Empty;
use cosmwasm_std::{coin, to_json_binary, Decimal, QueryRequest, Timestamp, Uint128, WasmQuery};
use cw_multi_test::Executor;
use factory_types::CustomPaymentError;
use minter_types::{Token, TokenDetails};

use minter_types::Config as MinterConfig;
use minter_types::QueryMsg;

use omniflix_minter_factory::msg::ExecuteMsg as FactoryExecuteMsg;

use crate::helpers::utils::{
    get_contract_address_from_res, return_factory_inst_message, return_minter_instantiate_msg,
};

use crate::{helpers::setup::setup, helpers::utils::query_onft_collection};
use omniflix_minter::error::ContractError as MinterContractError;
use omniflix_minter_factory::error::ContractError as MinterFactoryError;

#[test]
fn test_minter_creation() {
    let (
        mut app,
        test_addresses,
        minter_factory_code_id,
        minter_code_id,
        _round_whitelist_factory_code_id,
        _round_whitelist_code_id,
        _open_edition_minter_factory_code_id,
        _open_edition_minter_code_id,
        _multi_mint_open_edition_minter_code_id,
    ) = setup();
    let admin = test_addresses.admin;
    let creator = test_addresses.creator;
    let _collector = test_addresses.collector;

    let factory_inst_msg = return_factory_inst_message(minter_code_id);
    let factory_addr = app
        .instantiate_contract(
            minter_factory_code_id,
            admin.clone(),
            &factory_inst_msg,
            &[],
            "factory",
            None,
        )
        .unwrap();

    let minter_inst_msg = return_minter_instantiate_msg();
    let create_minter_msg = FactoryExecuteMsg::CreateMinter {
        msg: minter_inst_msg,
    };
    // Send no funds
    let error = app
        .execute_contract(
            creator.clone(),
            factory_addr.clone(),
            &create_minter_msg,
            &[],
        )
        .unwrap_err();

    let res = error.source().unwrap();
    let error = res.downcast_ref::<MinterFactoryError>().unwrap();
    assert_eq!(
        error,
        &MinterFactoryError::PaymentError(CustomPaymentError::InsufficientFunds {
            expected: [coin(1000000, "uflix"), coin(1000000, "uflix")].to_vec(),
            actual: [].to_vec()
        })
    );
    // Send incorrect denom
    let error = app
        .execute_contract(
            creator.clone(),
            factory_addr.clone(),
            &create_minter_msg,
            &[coin(1000000, "diffirent_denom")],
        )
        .unwrap_err();

    let res = error.source().unwrap();
    let error = res.downcast_ref::<MinterFactoryError>().unwrap();
    assert_eq!(
        error,
        &MinterFactoryError::PaymentError(CustomPaymentError::InsufficientFunds {
            expected: [coin(1000000, "uflix"), coin(1000000, "uflix")].to_vec(),
            actual: [coin(1000000, "diffirent_denom")].to_vec()
        })
    );
    // Send correct denom incorrect amount
    let error = app
        .execute_contract(
            creator.clone(),
            factory_addr.clone(),
            &create_minter_msg,
            &[coin(1000000, "uflix")],
        )
        .unwrap_err();

    let res = error.source().unwrap();
    let error = res.downcast_ref::<MinterFactoryError>().unwrap();
    assert_eq!(
        error,
        &MinterFactoryError::PaymentError(CustomPaymentError::InsufficientFunds {
            expected: [coin(1000000, "uflix"), coin(1000000, "uflix")].to_vec(),
            actual: [coin(1000000, "uflix")].to_vec()
        })
    );

    // Send 0 num tokens
    let mut minter_inst_msg = return_minter_instantiate_msg();
    minter_inst_msg.init.num_tokens = 0;
    let create_minter_msg = FactoryExecuteMsg::CreateMinter {
        msg: minter_inst_msg,
    };
    let error = app
        .execute_contract(
            creator.clone(),
            factory_addr.clone(),
            &create_minter_msg,
            &[coin(2000000, "uflix")],
        )
        .unwrap_err();
    let res = error.source().unwrap().source().unwrap();
    let error = res.downcast_ref::<MinterContractError>().unwrap();
    assert_eq!(error, &MinterContractError::InvalidNumTokens {});

    // Send royalty ratio more than 100%
    let mut minter_inst_msg = return_minter_instantiate_msg();
    minter_inst_msg
        .token_details
        .as_mut()
        .unwrap()
        .royalty_ratio = Decimal::percent(101);
    let create_minter_msg = FactoryExecuteMsg::CreateMinter {
        msg: minter_inst_msg,
    };
    let error = app
        .execute_contract(
            creator.clone(),
            factory_addr.clone(),
            &create_minter_msg,
            &[coin(2000000, "uflix")],
        )
        .unwrap_err();
    let res = error.source().unwrap().source().unwrap();
    let error = res.downcast_ref::<MinterContractError>().unwrap();
    assert_eq!(error, &MinterContractError::InvalidRoyaltyRatio {});

    // Send mint price 0
    let mut minter_inst_msg = return_minter_instantiate_msg();
    minter_inst_msg.init.mint_price.amount = Uint128::zero();
    let create_minter_msg = FactoryExecuteMsg::CreateMinter {
        msg: minter_inst_msg,
    };
    let error = app
        .execute_contract(
            creator.clone(),
            factory_addr.clone(),
            &create_minter_msg,
            &[coin(2000000, "uflix")],
        )
        .unwrap_err();
    let res = error.source().unwrap().source().unwrap();
    let error = res.downcast_ref::<MinterContractError>().unwrap();
    assert_eq!(error, &MinterContractError::InvalidMintPrice {});

    // Incorrect start time
    let mut minter_inst_msg = return_minter_instantiate_msg();
    minter_inst_msg.init.start_time = Timestamp::from_nanos(1_000 - 1);
    let create_minter_msg = FactoryExecuteMsg::CreateMinter {
        msg: minter_inst_msg,
    };
    let error = app
        .execute_contract(
            creator.clone(),
            factory_addr.clone(),
            &create_minter_msg,
            &[coin(2000000, "uflix")],
        )
        .unwrap_err();
    let res = error.source().unwrap().source().unwrap();
    let error = res.downcast_ref::<MinterContractError>().unwrap();
    assert_eq!(error, &MinterContractError::InvalidStartTime {});

    // Incorrect end time
    let mut minter_inst_msg = return_minter_instantiate_msg();
    minter_inst_msg.init.end_time = Some(minter_inst_msg.init.start_time.minus_nanos(1));
    let create_minter_msg = FactoryExecuteMsg::CreateMinter {
        msg: minter_inst_msg,
    };
    let error = app
        .execute_contract(
            creator.clone(),
            factory_addr.clone(),
            &create_minter_msg,
            &[coin(2000000, "uflix")],
        )
        .unwrap_err();
    let res = error.source().unwrap().source().unwrap();
    let error = res.downcast_ref::<MinterContractError>().unwrap();
    assert_eq!(error, &MinterContractError::InvalidEndTime {});

    // Happy path
    let minter_inst_msg = return_minter_instantiate_msg();
    let create_minter_msg = FactoryExecuteMsg::CreateMinter {
        msg: minter_inst_msg,
    };
    // Query balance of factory admin before
    let query_res = app
        .wrap()
        .query_balance(admin.clone(), "uflix".to_string())
        .unwrap();
    let uflix_before = query_res.amount;

    let res = app
        .execute_contract(
            admin.clone(),
            factory_addr,
            &create_minter_msg,
            &[coin(2000000, "uflix")],
        )
        .unwrap();
    // Query balance of factory admin after
    let query_res = app
        .wrap()
        .query_balance(admin.clone(), "uflix".to_string())
        .unwrap();
    let uflix_after = query_res.amount;
    assert_eq!(uflix_before - uflix_after, Uint128::from(1000000u128));

    let minter_address = get_contract_address_from_res(res.clone());
    let storage = app.storage();
    let collection = query_onft_collection(storage, minter_address.clone());
    assert_eq!(collection.denom.clone().unwrap().name, "name".to_string());
    assert_eq!(collection.denom.unwrap().id, "id".to_string());

    // Query config
    let config_data: MinterConfig = app
        .wrap()
        .query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: minter_address.clone(),
            msg: to_json_binary(&QueryMsg::<Empty>::Config {}).unwrap(),
        }))
        .unwrap();
    assert_eq!(config_data.per_address_limit, Some(1));
    assert_eq!(config_data.mint_price.denom, "uflix".to_string());
    assert_eq!(config_data.start_time, Timestamp::from_nanos(1000000000));
    assert_eq!(config_data.mint_price.amount, Uint128::from(1000000u128));

    let token_details: TokenDetails = app
        .wrap()
        .query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: minter_address.clone(),
            msg: to_json_binary(&QueryMsg::<Empty>::TokenDetails {}).unwrap(),
        }))
        .unwrap();
    assert_eq!(token_details.royalty_ratio, Decimal::percent(10));

    // Query mintable tokens
    let mintable_tokens_data: Vec<Token> = app
        .wrap()
        .query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: minter_address.clone(),
            msg: to_json_binary(&QueryMsg::Extension(
                omniflix_minter::msg::MinterExtensionQueryMsg::MintableTokens {},
            ))
            .unwrap(),
        }))
        .unwrap();
    assert_eq!(mintable_tokens_data.len(), 1000);
    // This is not a proper check but I am making sure list is randomized and is not starting from 1
    assert_ne!(mintable_tokens_data[0].token_id, 1.to_string());

    // Check total tokens remaining
    let total_tokens_remaining_data: u32 = app
        .wrap()
        .query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: minter_address.clone(),
            msg: to_json_binary(&QueryMsg::Extension(
                omniflix_minter::msg::MinterExtensionQueryMsg::TotalTokensRemaining {},
            ))
            .unwrap(),
        }))
        .unwrap();
    assert_eq!(total_tokens_remaining_data, 1000);
}
