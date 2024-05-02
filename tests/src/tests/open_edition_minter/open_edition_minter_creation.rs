#![cfg(test)]
use cosmwasm_std::{coin, Coin, Decimal, Timestamp, Uint128};

use cw_multi_test::Executor;
use factory_types::CustomPaymentError;
use minter_types::collection_details::CollectionDetails;
use minter_types::config::{Config, ConfigurationError};
use minter_types::token_details::TokenDetailsError;

use omniflix_open_edition_minter_factory::msg::ExecuteMsg as OpenEditionMinterFactoryExecuteMsg;

use crate::helpers::utils::get_contract_address_from_res;

use crate::helpers::mock_messages::factory_mock_messages::return_open_edition_minter_factory_inst_message;

use crate::helpers::mock_messages::oem_mock_messages::return_open_edition_minter_inst_msg;

use crate::helpers::utils::query_onft_collection;

use crate::helpers::setup::setup;

use minter_types::msg::QueryMsg as OpenEditionMinterQueryMsg;
use omniflix_open_edition_minter::msg::OEMQueryExtension;

use omniflix_open_edition_minter::error::ContractError as OpenEditionMinterError;

use omniflix_open_edition_minter_factory::error::ContractError as OpenEditionMinterFactoryError;

#[test]
fn open_edition_minter_creation() {
    let res = setup();
    let admin = res.test_accounts.admin;
    let creator = res.test_accounts.creator;
    let open_edition_minter_factory_code_id = res.open_edition_minter_factory_code_id;
    let open_edition_minter_code_id = res.open_edition_minter_code_id;
    let mut app = res.app;

    // Instantiate the oem minter factory
    let open_edition_minter_factory_instantiate_msg =
        return_open_edition_minter_factory_inst_message(open_edition_minter_code_id, None);

    let open_edition_minter_factory_address = app
        .instantiate_contract(
            open_edition_minter_factory_code_id,
            admin.clone(),
            &open_edition_minter_factory_instantiate_msg,
            &[],
            "Open Edition Minter Factory",
            None,
        )
        .unwrap();

    let open_edition_minter_instantiate_msg = return_open_edition_minter_inst_msg();
    let create_minter_msg = OpenEditionMinterFactoryExecuteMsg::CreateOpenEditionMinter {
        msg: open_edition_minter_instantiate_msg,
    };
    // Send no funds
    let res = app
        .execute_contract(
            creator.clone(),
            open_edition_minter_factory_address.clone(),
            &create_minter_msg,
            &[],
        )
        .unwrap_err();
    let err = res.source().unwrap();
    let error = err.downcast_ref::<OpenEditionMinterFactoryError>().unwrap();
    assert_eq!(
        OpenEditionMinterFactoryError::PaymentError(CustomPaymentError::InsufficientFunds {
            expected: [
                Coin {
                    denom: "uflix".to_string(),
                    amount: Uint128::from(1000000u128)
                },
                Coin {
                    denom: "uflix".to_string(),
                    amount: Uint128::from(1000000u128)
                }
            ]
            .to_vec(),
            actual: vec![]
        }),
        *error
    );

    // Send incorrect funds
    let res = app
        .execute_contract(
            creator.clone(),
            open_edition_minter_factory_address.clone(),
            &create_minter_msg,
            &[coin(1000000, "incorrect_denom")],
        )
        .unwrap_err();
    let err = res.source().unwrap();
    let error = err.downcast_ref::<OpenEditionMinterFactoryError>().unwrap();
    assert_eq!(
        *error,
        OpenEditionMinterFactoryError::PaymentError(CustomPaymentError::InsufficientFunds {
            expected: [
                Coin {
                    denom: "uflix".to_string(),
                    amount: Uint128::from(1000000u128)
                },
                Coin {
                    denom: "uflix".to_string(),
                    amount: Uint128::from(1000000u128)
                }
            ]
            .to_vec(),
            actual: vec![coin(1000000, "incorrect_denom"),]
        }),
    );

    // Send incorrect amount
    let res = app
        .execute_contract(
            creator.clone(),
            open_edition_minter_factory_address.clone(),
            &create_minter_msg,
            &[coin(1000000, "uflix")],
        )
        .unwrap_err();
    let err = res.source().unwrap();
    let error = err.downcast_ref::<OpenEditionMinterFactoryError>().unwrap();
    assert_eq!(
        *error,
        OpenEditionMinterFactoryError::PaymentError(CustomPaymentError::InsufficientFunds {
            expected: [
                Coin {
                    denom: "uflix".to_string(),
                    amount: Uint128::from(1000000u128)
                },
                Coin {
                    denom: "uflix".to_string(),
                    amount: Uint128::from(1000000u128)
                }
            ]
            .to_vec(),
            actual: vec![coin(1000000, "uflix"),]
        }),
    );

    // Send zero token limit
    let mut open_edition_minter_instantiate_msg = return_open_edition_minter_inst_msg();
    let mut init = open_edition_minter_instantiate_msg.init.unwrap();
    init.num_tokens = Some(0);
    open_edition_minter_instantiate_msg.init = Some(init);
    let create_minter_msg = OpenEditionMinterFactoryExecuteMsg::CreateOpenEditionMinter {
        msg: open_edition_minter_instantiate_msg,
    };
    let res = app
        .execute_contract(
            creator.clone(),
            open_edition_minter_factory_address.clone(),
            &create_minter_msg,
            &[coin(2000000, "uflix")],
        )
        .unwrap_err();

    let err = res.source().unwrap().source().unwrap();

    let error = err.downcast_ref::<OpenEditionMinterError>().unwrap();
    assert_eq!(
        OpenEditionMinterError::ConfigurationError(ConfigurationError::InvalidNumberOfTokens {}),
        *error
    );

    // Send zero per address limit
    let mut open_edition_minter_instantiate_msg = return_open_edition_minter_inst_msg();
    let mut init = open_edition_minter_instantiate_msg.init.unwrap();
    init.per_address_limit = Some(0);
    open_edition_minter_instantiate_msg.init = Some(init);
    let create_minter_msg = OpenEditionMinterFactoryExecuteMsg::CreateOpenEditionMinter {
        msg: open_edition_minter_instantiate_msg,
    };
    let res = app
        .execute_contract(
            creator.clone(),
            open_edition_minter_factory_address.clone(),
            &create_minter_msg,
            &[coin(2000000, "uflix")],
        )
        .unwrap_err();

    let err = res.source().unwrap().source().unwrap();

    let error = err.downcast_ref::<OpenEditionMinterError>().unwrap();
    assert_eq!(
        OpenEditionMinterError::ConfigurationError(ConfigurationError::InvalidPerAddressLimit {}),
        *error
    );

    // Send incorrect royalty ratio
    let mut open_edition_minter_instantiate_msg = return_open_edition_minter_inst_msg();
    open_edition_minter_instantiate_msg
        .token_details
        .as_mut()
        .unwrap()
        .royalty_ratio = Decimal::percent(101);
    let create_minter_msg = OpenEditionMinterFactoryExecuteMsg::CreateOpenEditionMinter {
        msg: open_edition_minter_instantiate_msg,
    };
    let res = app
        .execute_contract(
            creator.clone(),
            open_edition_minter_factory_address.clone(),
            &create_minter_msg,
            &[coin(2000000, "uflix")],
        )
        .unwrap_err();

    let err = res.source().unwrap().source().unwrap();

    let error = err.downcast_ref::<OpenEditionMinterError>().unwrap();
    assert_eq!(
        OpenEditionMinterError::TokenDetailsError(TokenDetailsError::InvalidRoyaltyRatio {}),
        *error
    );
    // Send too long description
    let mut open_edition_minter_instantiate_msg = return_open_edition_minter_inst_msg();
    open_edition_minter_instantiate_msg
        .token_details
        .as_mut()
        .unwrap()
        .description = Some("a".repeat(5001));
    let create_minter_msg = OpenEditionMinterFactoryExecuteMsg::CreateOpenEditionMinter {
        msg: open_edition_minter_instantiate_msg,
    };
    let res = app
        .execute_contract(
            creator.clone(),
            open_edition_minter_factory_address.clone(),
            &create_minter_msg,
            &[coin(2000000, "uflix")],
        )
        .unwrap_err();

    let err = res.source().unwrap().source().unwrap();

    let error = err.downcast_ref::<OpenEditionMinterError>().unwrap();
    assert_eq!(
        OpenEditionMinterError::TokenDetailsError(TokenDetailsError::TokenDescriptionTooLong {}),
        *error
    );

    // Send 0 mint price this should not fail because mint price can be set to zero on open edition minter
    let mut open_edition_minter_instantiate_msg = return_open_edition_minter_inst_msg();
    let mut init = open_edition_minter_instantiate_msg.init.unwrap();
    init.mint_price.amount = Uint128::zero();
    open_edition_minter_instantiate_msg.init = Some(init);
    let create_minter_msg = OpenEditionMinterFactoryExecuteMsg::CreateOpenEditionMinter {
        msg: open_edition_minter_instantiate_msg,
    };
    let _res = app
        .execute_contract(
            creator.clone(),
            open_edition_minter_factory_address.clone(),
            &create_minter_msg,
            &[coin(2000000, "uflix")],
        )
        .unwrap();

    // Send incorrect start time
    let mut open_edition_minter_instantiate_msg = return_open_edition_minter_inst_msg();
    let mut init = open_edition_minter_instantiate_msg.init.unwrap();
    init.start_time = Timestamp::from_nanos(0);
    open_edition_minter_instantiate_msg.init = Some(init);
    let create_minter_msg = OpenEditionMinterFactoryExecuteMsg::CreateOpenEditionMinter {
        msg: open_edition_minter_instantiate_msg,
    };
    let res = app
        .execute_contract(
            creator.clone(),
            open_edition_minter_factory_address.clone(),
            &create_minter_msg,
            &[coin(2000000, "uflix")],
        )
        .unwrap_err();

    let err = res.source().unwrap().source().unwrap();

    let error = err.downcast_ref::<OpenEditionMinterError>().unwrap();
    assert_eq!(
        OpenEditionMinterError::ConfigurationError(ConfigurationError::InvalidStartTime {}),
        *error
    );

    // Check factory admin balance before happy path
    let query_res = app
        .wrap()
        .query_balance(admin.clone(), "uflix".to_string())
        .unwrap();
    let uflix_before = query_res.amount;

    // Create a minter
    let open_edition_minter_instantiate_msg = return_open_edition_minter_inst_msg();
    let create_minter_msg = OpenEditionMinterFactoryExecuteMsg::CreateOpenEditionMinter {
        msg: open_edition_minter_instantiate_msg,
    };
    let res = app
        .execute_contract(
            creator.clone(),
            open_edition_minter_factory_address.clone(),
            &create_minter_msg,
            &[coin(2000000, "uflix")],
        )
        .unwrap();
    let open_edition_minter_address = get_contract_address_from_res(res);

    // Check factory admin balance after happy path
    let query_res = app
        .wrap()
        .query_balance(admin.clone(), "uflix".to_string())
        .unwrap();
    let uflix_after = query_res.amount;

    // We are collecting fee as expected
    assert_eq!(uflix_after - uflix_before, Uint128::from(1000000u128));

    let config_res: Config = app
        .wrap()
        .query_wasm_smart(
            open_edition_minter_address.clone(),
            &OpenEditionMinterQueryMsg::<OEMQueryExtension>::Config {},
        )
        .unwrap();
    assert_eq!(
        config_res,
        Config {
            end_time: Some(Timestamp::from_nanos(2_000_000_000)),
            start_time: Timestamp::from_nanos(1_000_000_000),
            mint_price: Coin {
                denom: "uflix".to_string(),
                amount: Uint128::from(1000000u128)
            },
            per_address_limit: Some(1),
            whitelist_address: None,
            num_tokens: Some(1000)
        }
    );

    // Query the minter
    let query_msg = OpenEditionMinterQueryMsg::Extension(OEMQueryExtension::TokensRemaining {});

    let tokens_remaining_res: u32 = app
        .wrap()
        .query_wasm_smart(open_edition_minter_address.clone(), &query_msg)
        .unwrap();

    assert_eq!(tokens_remaining_res, 1000);

    // Query the minter
    let query_msg = OpenEditionMinterQueryMsg::<OEMQueryExtension>::TotalMintedCount {};

    let total_minted_count_res: u32 = app
        .wrap()
        .query_wasm_smart(open_edition_minter_address.clone(), &query_msg)
        .unwrap();

    assert_eq!(total_minted_count_res, 0);

    // Query the minter
    let query_msg = OpenEditionMinterQueryMsg::<OEMQueryExtension>::Collection {};

    let collection_res: CollectionDetails = app
        .wrap()
        .query_wasm_smart(open_edition_minter_address.clone(), &query_msg)
        .unwrap();
    assert_eq!(
        collection_res,
        CollectionDetails {
            collection_name: "name".to_string(),
            description: Some("description".to_string()),
            preview_uri: Some("preview_uri".to_string()),
            schema: Some("schema".to_string()),
            symbol: "symbol".to_string(),
            id: "id".to_string(),
            uri: Some("uri".to_string()),
            uri_hash: Some("uri_hash".to_string()),
            data: Some("data".to_string()),
            royalty_receivers: None
        }
    );
    let collection = query_onft_collection(app.storage(), open_edition_minter_address.clone());

    assert_eq!(collection.denom.clone().unwrap().name, "name".to_string());
    assert_eq!(
        collection.denom.unwrap().description,
        "description".to_string()
    );
}
