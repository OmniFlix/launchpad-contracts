#![cfg(test)]
use cosmwasm_std::Decimal;
use cosmwasm_std::{coin, Addr, BlockInfo, Timestamp};
use cw_multi_test::Executor;
use minter_types::msg::QueryMsg as CommonMinterQueryMsg;
use minter_types::types::TokenDetails;
use minter_types::types::{CollectionDetails, Config, UserDetails};
use omniflix_multi_mint_open_edition_minter::error::ContractError as MultiMintOpenEditionMinterContractError;
use omniflix_multi_mint_open_edition_minter::msg::ExecuteMsg as MultiMintOpenEditionMinterExecuteMsg;
use omniflix_multi_mint_open_edition_minter::msg::QueryMsgExtension as MultiMintOpenEditionMinterQueryMsgExtension;

use omniflix_multi_mint_open_edition_minter::drop::{Drop, DropParams};
use omniflix_open_edition_minter_factory::error::ContractError as OpenEditionMinterFactoryError;
use omniflix_open_edition_minter_factory::msg::{
    ExecuteMsg as OpenEditionMinterFactoryExecuteMsg, MultiMinterCreateMsg,
    MultiMinterInitExtention,
};

type MultiMintOpenEditionMinterQueryMsg =
    CommonMinterQueryMsg<MultiMintOpenEditionMinterQueryMsgExtension>;

use crate::helpers::mock_messages::factory_mock_messages::return_open_edition_minter_factory_inst_message;
use crate::helpers::utils::get_contract_address_from_res;

use crate::helpers::setup::setup;
use crate::tests::multi_mint_open_edition_minter;

#[test]
fn test_multi_mint_oem_creation() {
    let res = setup();
    let admin = res.test_accounts.admin;
    let creator = res.test_accounts.creator;
    let collector = res.test_accounts.collector;
    let open_edition_minter_factory_code_id = res.open_edition_minter_factory_code_id;
    let multi_mint_open_edition_minter_code_id = res.multi_mint_open_edition_minter_code_id;
    // Instantiate the minter factory
    let open_edition_minter_factory_instantiate_msg =
        return_open_edition_minter_factory_inst_message(
            open_edition_minter_factory_code_id,
            Some(multi_mint_open_edition_minter_code_id),
        );

    let mut app = res.app;

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
    let collection_details = CollectionDetails {
        collection_name: "Multi mint test".to_string(),
        description: Some("COLLECTION DESCRIPTION".to_string()),
        preview_uri: Some("Preview uri of COLLECTION".to_string()),
        schema: Some("Some schema of collection".to_string()),
        symbol: "MMOEM".to_string(),
        id: "MMOEM test 1".to_string(),
        uri: Some("Some uri".to_string()),
        uri_hash: Some("uri_hash".to_string()),
        data: Some("data".to_string()),
        royalty_receivers: None,
    };
    let init = MultiMinterInitExtention {
        admin: creator.to_string(),
        payment_collector: Some(creator.to_string()),
    };

    let multi_minter_inst_msg = MultiMinterCreateMsg {
        collection_details: collection_details.clone(),
        init,
        token_details: None,
    };
    // Send no funds
    let res = app
        .execute_contract(
            creator.clone(),
            open_edition_minter_factory_address.clone(),
            &OpenEditionMinterFactoryExecuteMsg::CreateMultiMintOpenEditionMinter {
                msg: multi_minter_inst_msg.clone(),
            },
            &[],
        )
        .unwrap_err();
    let res = res.source().unwrap();
    let error = res.downcast_ref::<OpenEditionMinterFactoryError>().unwrap();
    assert_eq!(
        error,
        &OpenEditionMinterFactoryError::PaymentError(
            factory_types::CustomPaymentError::InsufficientFunds {
                expected: vec![coin(1000000, "uflix"), coin(1000000, "uflix")],
                actual: vec![]
            }
        )
    );
    // Send incorrect funds
    let res = app
        .execute_contract(
            creator.clone(),
            open_edition_minter_factory_address.clone(),
            &OpenEditionMinterFactoryExecuteMsg::CreateMultiMintOpenEditionMinter {
                msg: multi_minter_inst_msg.clone(),
            },
            &[coin(1000000, "incorrect_denom")],
        )
        .unwrap_err();
    let err = res.source().unwrap();
    let error = err.downcast_ref::<OpenEditionMinterFactoryError>().unwrap();
    assert_eq!(
        *error,
        OpenEditionMinterFactoryError::PaymentError(
            factory_types::CustomPaymentError::InsufficientFunds {
                expected: vec![coin(1000000, "uflix"), coin(1000000, "uflix")],
                actual: vec![coin(1000000, "incorrect_denom"),]
            }
        )
    );

    // Send incorrect amount
    let res = app
        .execute_contract(
            creator.clone(),
            open_edition_minter_factory_address.clone(),
            &OpenEditionMinterFactoryExecuteMsg::CreateMultiMintOpenEditionMinter {
                msg: multi_minter_inst_msg.clone(),
            },
            &[coin(1000000, "uflix")],
        )
        .unwrap_err();
    let err = res.source().unwrap();
    let error = err.downcast_ref::<OpenEditionMinterFactoryError>().unwrap();
    assert_eq!(
        *error,
        OpenEditionMinterFactoryError::PaymentError(
            factory_types::CustomPaymentError::InsufficientFunds {
                expected: vec![coin(1000000, "uflix"), coin(1000000, "uflix")],
                actual: vec![coin(1000000, "uflix"),]
            }
        )
    );

    // Happy path
    let res = app
        .execute_contract(
            creator.clone(),
            open_edition_minter_factory_address.clone(),
            &OpenEditionMinterFactoryExecuteMsg::CreateMultiMintOpenEditionMinter {
                msg: multi_minter_inst_msg.clone(),
            },
            &[coin(2000000, "uflix")],
        )
        .unwrap();
    let multi_mint_open_edition_minter_address = get_contract_address_from_res(res);

    // Query the minter
    let collection: CollectionDetails = app
        .wrap()
        .query_wasm_smart(
            &multi_mint_open_edition_minter_address,
            &MultiMintOpenEditionMinterQueryMsg::Collection {},
        )
        .unwrap();
    assert_eq!(collection, collection_details);

    let active_drop_id: u32 = app
        .wrap()
        .query_wasm_smart(
            &multi_mint_open_edition_minter_address,
            &MultiMintOpenEditionMinterQueryMsg::Extension(
                MultiMintOpenEditionMinterQueryMsgExtension::ActiveDropId {},
            ),
        )
        .unwrap();
    assert_eq!(active_drop_id, 0);

    let minted_count: u32 = app
        .wrap()
        .query_wasm_smart(
            &multi_mint_open_edition_minter_address,
            &MultiMintOpenEditionMinterQueryMsg::TotalMintedCount {},
        )
        .unwrap();
    assert_eq!(minted_count, 0);

    let drops: Vec<Drop> = app
        .wrap()
        .query_wasm_smart(
            &multi_mint_open_edition_minter_address,
            &MultiMintOpenEditionMinterQueryMsg::Extension(
                MultiMintOpenEditionMinterQueryMsgExtension::AllDrops {},
            ),
        )
        .unwrap();
    assert_eq!(drops.len(), 0);
}
