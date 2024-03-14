#![cfg(test)]
use cosmwasm_std::coin;
use cw_multi_test::Executor;
use minter_types::types::CollectionDetails;
use omniflix_open_edition_minter_factory::error::ContractError as OpenEditionMinterFactoryError;

use omniflix_open_edition_minter_factory::msg::{
    ExecuteMsg as OpenEditionMinterFactoryExecuteMsg, MultiMinterCreateMsg,
    MultiMinterInitExtention, QueryMsg as OpenEditionMinterFactoryQueryMsg,
};
use pauser::PauseError;

use crate::helpers::mock_messages::factory_mock_messages::return_open_edition_minter_factory_inst_message;

use crate::helpers::setup::setup;

#[test]
fn paused_mm_oem_factory() {
    let res = setup();
    let admin = res.test_accounts.admin;
    let creator = res.test_accounts.creator;
    let _collector = res.test_accounts.collector;
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
    // Ensure that the factory is not paused
    let query_msg = OpenEditionMinterFactoryQueryMsg::IsPaused {};
    let is_paused: bool = app
        .wrap()
        .query_wasm_smart(open_edition_minter_factory_address.clone(), &query_msg)
        .unwrap();
    assert!(!is_paused);

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

    let _res = app
        .execute_contract(
            creator.clone(),
            open_edition_minter_factory_address.clone(),
            &OpenEditionMinterFactoryExecuteMsg::CreateMultiMintOpenEditionMinter {
                msg: multi_minter_inst_msg.clone(),
            },
            &[coin(20_000_000, "uflix".to_string())],
        )
        .unwrap();

    // Pause the factory
    let pause_msg = OpenEditionMinterFactoryExecuteMsg::Pause {};
    let _res = app
        .execute_contract(
            admin.clone(),
            open_edition_minter_factory_address.clone(),
            &pause_msg,
            &[],
        )
        .unwrap();

    // Ensure that the factory is paused
    let is_paused: bool = app
        .wrap()
        .query_wasm_smart(open_edition_minter_factory_address.clone(), &query_msg)
        .unwrap();
    assert!(is_paused);

    // Ensure that the minter cannot be created
    let error = app
        .execute_contract(
            creator.clone(),
            open_edition_minter_factory_address.clone(),
            &OpenEditionMinterFactoryExecuteMsg::CreateMultiMintOpenEditionMinter {
                msg: multi_minter_inst_msg.clone(),
            },
            &[coin(20_000_000, "uflix".to_string())],
        )
        .unwrap_err();
    let err = error.source().unwrap();
    let error = err.downcast_ref::<OpenEditionMinterFactoryError>().unwrap();
    assert_eq!(
        *error,
        OpenEditionMinterFactoryError::Pause(PauseError::Paused {})
    );
}
