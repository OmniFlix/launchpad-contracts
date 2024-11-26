#![cfg(test)]
use cosmwasm_std::Decimal;
use cosmwasm_std::{coin, Addr, Timestamp};
use cw_multi_test::Executor;
use minter_types::collection_details::CollectionDetails;
use minter_types::config::Config;
use minter_types::msg::QueryMsg as CommonMinterQueryMsg;
use minter_types::token_details::TokenDetails;
use minter_types::types::AuthDetails;
use omniflix_multi_mint_open_edition_minter::error::ContractError as MultiMintOpenEditionMinterContractError;
use omniflix_multi_mint_open_edition_minter::msg::ExecuteMsg as MultiMintOpenEditionMinterExecuteMsg;
use omniflix_multi_mint_open_edition_minter::msg::QueryMsgExtension as MultiMintOpenEditionMinterQueryMsgExtension;

use omniflix_open_edition_minter_factory::msg::{
    ExecuteMsg as OpenEditionMinterFactoryExecuteMsg, MultiMinterCreateMsg,
};
use pauser::PauseError;

type MultiMintOpenEditionMinterQueryMsg =
    CommonMinterQueryMsg<MultiMintOpenEditionMinterQueryMsgExtension>;

use crate::helpers::mock_messages::factory_mock_messages::return_open_edition_minter_factory_inst_message;
use crate::helpers::utils::get_contract_address_from_res;

use crate::helpers::setup::setup;

#[test]
fn paused_mm_oem() {
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

    let multi_minter_inst_msg = MultiMinterCreateMsg {
        collection_details,
        token_details: None,
        auth_details: AuthDetails {
            admin: Addr::unchecked("creator".to_string()),
            payment_collector: Addr::unchecked("creator".to_string()),
        },
        init: Default::default(),
    };

    let res = app
        .execute_contract(
            creator.clone(),
            open_edition_minter_factory_address.clone(),
            &OpenEditionMinterFactoryExecuteMsg::CreateMultiMintOpenEditionMinter {
                msg: multi_minter_inst_msg,
            },
            &[coin(2000000, "uflix")],
        )
        .unwrap();
    let multi_minter_addr = get_contract_address_from_res(res);

    // Create first mint_instance
    let token_details = TokenDetails {
        token_name: "MintInstance number 1".to_string(),
        description: Some("MintInstance number 1 description".to_string()),
        preview_uri: Some("MintInstance number 1 prev uri".to_string()),
        base_token_uri: "MintInstance number 1 base_token_uri".to_string(),
        transferable: true,
        royalty_ratio: Decimal::percent(10),
        extensible: true,
        nsfw: false,
        data: Some("MintInstance number 1 data".to_string()),
    };
    let config = Config {
        mint_price: coin(5_000_000, "uflix"),
        start_time: Timestamp::from_nanos(10_000_000),
        end_time: Some(Timestamp::from_nanos(50_500_000)),
        per_address_limit: Some(100),
        whitelist_address: None,
        num_tokens: Some(100),
    };
    let _res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(multi_minter_addr.clone()),
            &MultiMintOpenEditionMinterExecuteMsg::CreateMintInstance {
                config,
                token_details,
            },
            &[coin(2000000, "uflix")],
        )
        .unwrap();

    let active_mint_instance: u32 = app
        .wrap()
        .query_wasm_smart(
            multi_minter_addr.clone(),
            &MultiMintOpenEditionMinterQueryMsg::Extension(
                MultiMintOpenEditionMinterQueryMsgExtension::ActiveMintInstanceId {},
            ),
        )
        .unwrap();
    assert_eq!(active_mint_instance, 1);

    // Ensure that minter is not paused
    let query_msg = MultiMintOpenEditionMinterQueryMsg::IsPaused {};
    let is_paused: bool = app
        .wrap()
        .query_wasm_smart(multi_minter_addr.clone(), &query_msg)
        .unwrap();
    assert!(!is_paused);

    // Pause the minter
    // Only creator can pause the minter
    // Try to pause the minter with a different account
    let pause_msg = MultiMintOpenEditionMinterExecuteMsg::Pause {};
    let error = app
        .execute_contract(
            collector.clone(),
            Addr::unchecked(multi_minter_addr.clone()),
            &pause_msg,
            &[],
        )
        .unwrap_err();
    let error = error
        .downcast_ref::<MultiMintOpenEditionMinterContractError>()
        .unwrap();
    assert_eq!(
        *error,
        MultiMintOpenEditionMinterContractError::Pause(PauseError::Unauthorized {
            sender: collector.clone()
        })
    );
    // Pause the minter with the creator
    let _res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(multi_minter_addr.clone()),
            &pause_msg,
            &[],
        )
        .unwrap();

    // Ensure that the minter is paused
    let is_paused: bool = app
        .wrap()
        .query_wasm_smart(multi_minter_addr.clone(), &query_msg)
        .unwrap();
    assert!(is_paused);

    // Ensure that the minter can not mint
    let mint_msg = MultiMintOpenEditionMinterExecuteMsg::Mint {
        mint_instance_id: None,
    };

    let error = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(multi_minter_addr.clone()),
            &mint_msg,
            &[coin(2000000, "uflix")],
        )
        .unwrap_err();
    let error = error
        .downcast_ref::<MultiMintOpenEditionMinterContractError>()
        .unwrap();
    assert_eq!(
        *error,
        MultiMintOpenEditionMinterContractError::Pause(PauseError::Paused {})
    );
    let admin_mint_msg = MultiMintOpenEditionMinterExecuteMsg::MintAdmin {
        recipient: creator.to_string(),
        mint_instance_id: None,
    };
    // Ensure that MintAdmin cannot mint
    let error = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(multi_minter_addr.clone()),
            &admin_mint_msg,
            &[],
        )
        .unwrap_err();

    let error = error
        .downcast_ref::<MultiMintOpenEditionMinterContractError>()
        .unwrap();
    assert_eq!(
        *error,
        MultiMintOpenEditionMinterContractError::Pause(PauseError::Paused {})
    );
    // Try unpausing the minter with a non pauser
    let unpause_msg = MultiMintOpenEditionMinterExecuteMsg::Unpause {};
    let error = app
        .execute_contract(
            collector.clone(),
            Addr::unchecked(multi_minter_addr.clone()),
            &unpause_msg,
            &[],
        )
        .unwrap_err();
    let error = error
        .downcast_ref::<MultiMintOpenEditionMinterContractError>()
        .unwrap();
    assert_eq!(
        *error,
        MultiMintOpenEditionMinterContractError::Pause(PauseError::Unauthorized {
            sender: collector.clone()
        })
    );

    // Unpause the minter
    let unpause_msg = MultiMintOpenEditionMinterExecuteMsg::Unpause {};
    let _res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(multi_minter_addr.clone()),
            &unpause_msg,
            &[],
        )
        .unwrap();

    // Ensure that the minter is unpaused
    let is_paused: bool = app
        .wrap()
        .query_wasm_smart(multi_minter_addr.clone(), &query_msg)
        .unwrap();
    assert!(!is_paused);
}
