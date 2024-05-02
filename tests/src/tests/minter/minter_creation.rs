#![cfg(test)]
use cosmwasm_std::{coin, to_json_binary, Decimal, QueryRequest, Timestamp, Uint128, WasmQuery};
use cosmwasm_std::{BlockInfo, Empty};
use cw_multi_test::Executor;
use factory_types::CustomPaymentError;
use minter_types::msg::QueryMsg;

use minter_types::config::{Config as MinterConfig, ConfigurationError};
use minter_types::token_details::{Token, TokenDetails, TokenDetailsError};

use omniflix_minter_factory::msg::ExecuteMsg as FactoryExecuteMsg;
use whitelist_types::CreateWhitelistMsg;

use crate::helpers::mock_messages::factory_mock_messages::{
    return_minter_factory_inst_message, return_round_whitelist_factory_inst_message,
};
use crate::helpers::mock_messages::minter_mock_messages::return_minter_instantiate_msg;
use crate::helpers::mock_messages::whitelist_mock_messages::return_rounds;
use crate::helpers::utils::get_contract_address_from_res;

use crate::{helpers::setup::setup, helpers::utils::query_onft_collection};
use omniflix_minter::error::ContractError as MinterContractError;
use omniflix_minter_factory::error::ContractError as MinterFactoryError;

#[test]
fn minter_creation() {
    let res = setup();
    let admin = res.test_accounts.admin;
    let creator = res.test_accounts.creator;
    let minter_factory_code_id = res.minter_factory_code_id;
    let minter_code_id = res.minter_code_id;
    let mut app = res.app;

    let factory_inst_msg = return_minter_factory_inst_message(minter_code_id);
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
    let mut init = minter_inst_msg.init.unwrap();
    init.num_tokens = 0;
    minter_inst_msg.init = Some(init);
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
    assert_eq!(
        error,
        &MinterContractError::ConfigurationError(ConfigurationError::InvalidNumberOfTokens {})
    );

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
    assert_eq!(
        error,
        &MinterContractError::TokenDetailsError(TokenDetailsError::InvalidRoyaltyRatio {})
    );

    // Incorrect start time
    let mut minter_inst_msg = return_minter_instantiate_msg();
    let mut init = minter_inst_msg.init.unwrap();
    init.start_time = Timestamp::from_nanos(1_000 - 1);
    minter_inst_msg.init = Some(init);
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
    assert_eq!(
        error,
        &MinterContractError::ConfigurationError(ConfigurationError::InvalidStartTime {})
    );

    // Incorrect end time
    let mut minter_inst_msg = return_minter_instantiate_msg();
    let mut init = minter_inst_msg.init.unwrap();
    init.end_time = Some(init.start_time.minus_nanos(1));
    minter_inst_msg.init = Some(init);
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
    assert_eq!(
        error,
        &MinterContractError::ConfigurationError(ConfigurationError::InvalidEndTime {})
    );

    // Send none token details
    let mut minter_inst_msg = return_minter_instantiate_msg();
    minter_inst_msg.token_details = None;
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
    assert_eq!(error, &MinterContractError::InvalidTokenDetails {});

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

#[test]
fn test_minter_creation_with_whitelist() {
    let res = setup();
    let admin = res.test_accounts.admin;
    let _creator = res.test_accounts.creator;
    let minter_factory_code_id = res.minter_factory_code_id;
    let minter_code_id = res.minter_code_id;
    let round_whitelist_code_id = res.round_whitelist_code_id;
    let round_whitelist_factory_code_id = res.round_whitelist_factory_code_id;
    let mut app = res.app;

    let factory_inst_msg = return_minter_factory_inst_message(minter_code_id);
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

    let round_whitelist_factory_inst_msg =
        return_round_whitelist_factory_inst_message(round_whitelist_code_id);
    let round_whitelist_factory_addr = app
        .instantiate_contract(
            round_whitelist_factory_code_id,
            admin.clone(),
            &round_whitelist_factory_inst_msg,
            &[],
            "round_whitelist_factory",
            None,
        )
        .unwrap();
    let rounds = return_rounds();
    let round_1_start = rounds.clone()[0].start_time;
    let round_whitelist_inst_msg = CreateWhitelistMsg {
        admin: admin.to_string(),
        rounds: rounds.clone(),
    };
    let create_round_whitelist_msg =
        omniflix_round_whitelist_factory::msg::ExecuteMsg::CreateWhitelist {
            msg: round_whitelist_inst_msg,
        };
    // Create a whitelist
    let res = app
        .execute_contract(
            admin.clone(),
            round_whitelist_factory_addr,
            &create_round_whitelist_msg,
            &[coin(1000000, "uflix")],
        )
        .unwrap();
    let whitelist_address = get_contract_address_from_res(res);

    // Try creating a minter with already active whitelist
    let mut minter_inst_msg = return_minter_instantiate_msg();
    let mut init = minter_inst_msg.init.unwrap();
    init.whitelist_address = Some(whitelist_address.clone());
    minter_inst_msg.init = Some(init);
    let create_minter_msg = FactoryExecuteMsg::CreateMinter {
        msg: minter_inst_msg,
    };

    // Set time to round 1 start time
    app.set_block(BlockInfo {
        time: round_1_start,
        height: 1,
        chain_id: "".to_string(),
    });

    let res = app
        .execute_contract(
            admin.clone(),
            factory_addr.clone(),
            &create_minter_msg,
            &[coin(2000000, "uflix")],
        )
        .unwrap_err();
    let res = res.source().unwrap().source().unwrap();
    let error = res.downcast_ref::<MinterContractError>().unwrap();
    assert_eq!(error, &MinterContractError::WhitelistAlreadyActive {});

    // Reset time to default
    app.set_block(BlockInfo {
        time: Timestamp::from_nanos(1_000),
        height: 1,
        chain_id: "".to_string(),
    });

    // Create a minter with inactive whitelist
    let mut minter_inst_msg = return_minter_instantiate_msg();
    let mut init = minter_inst_msg.init.unwrap();
    init.whitelist_address = Some(whitelist_address.clone());
    minter_inst_msg.init = Some(init);
    let create_minter_msg = FactoryExecuteMsg::CreateMinter {
        msg: minter_inst_msg,
    };
    let _res = app
        .execute_contract(
            admin.clone(),
            factory_addr.clone(),
            &create_minter_msg,
            &[coin(2000000, "uflix")],
        )
        .unwrap();
}
