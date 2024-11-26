#![cfg(test)]
use cosmwasm_std::{coin, Addr, BlockInfo, Timestamp};
use cosmwasm_std::{Decimal, StdError};
use cw_multi_test::Executor;
use minter_types::collection_details::CollectionDetails;
use minter_types::config::{Config, ConfigurationError};
use minter_types::msg::QueryMsg as CommonMinterQueryMsg;
use minter_types::token_details::{TokenDetails, TokenDetailsError};

use minter_types::types::{AuthDetails, UserDetails};

use omniflix_multi_mint_open_edition_minter::error::ContractError as MultiMintOpenEditionMinterContractError;
use omniflix_multi_mint_open_edition_minter::msg::ExecuteMsg as MultiMintOpenEditionMinterExecuteMsg;
use omniflix_multi_mint_open_edition_minter::msg::QueryMsgExtension as MultiMintOpenEditionMinterQueryMsgExtension;

use omniflix_multi_mint_open_edition_minter::mint_instance::{MintInstance, MintInstanceParams};
use omniflix_open_edition_minter_factory::msg::{
    ExecuteMsg as OpenEditionMinterFactoryExecuteMsg, MultiMinterCreateMsg,
};

type MultiMintOpenEditionMinterQueryMsg =
    CommonMinterQueryMsg<MultiMintOpenEditionMinterQueryMsgExtension>;

use crate::helpers::mock_messages::factory_mock_messages::return_open_edition_minter_factory_inst_message;
use crate::helpers::utils::get_contract_address_from_res;

use crate::helpers::setup::setup;

#[test]
fn remove_mint_instance() {
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
    let active_mint_instance: u32 = app
        .wrap()
        .query_wasm_smart(
            multi_minter_addr.clone(),
            &MultiMintOpenEditionMinterQueryMsg::Extension(
                MultiMintOpenEditionMinterQueryMsgExtension::ActiveMintInstanceId {},
            ),
        )
        .unwrap();
    assert_eq!(active_mint_instance, 0);
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

    // Query all the mint_instances
    let mint_instances: Result<Vec<(u32, MintInstance)>, _> = app.wrap().query_wasm_smart(
        multi_minter_addr.clone(),
        &MultiMintOpenEditionMinterQueryMsg::Extension(
            MultiMintOpenEditionMinterQueryMsgExtension::AllMintInstances {},
        ),
    );
    assert_eq!(mint_instances.as_ref().unwrap().len(), 1);
    // Check mint_instance id
    assert_eq!(mint_instances.as_ref().unwrap()[0].0, 1);

    // Set time to public sale
    app.set_block(BlockInfo {
        time: Timestamp::from_nanos(20_000_000),
        height: 1,
        chain_id: "cosmos".to_string(),
    });

    // Mint token for the mint_instance
    app.execute_contract(
        collector.clone(),
        Addr::unchecked(multi_minter_addr.clone()),
        &MultiMintOpenEditionMinterExecuteMsg::Mint {
            mint_instance_id: None,
        },
        &[coin(5_000_000, "uflix")],
    )
    .unwrap();
    // Query all the mint_instances
    let mint_instances: Result<Vec<(u32, MintInstance)>, _> = app.wrap().query_wasm_smart(
        multi_minter_addr.clone(),
        &MultiMintOpenEditionMinterQueryMsg::Extension(
            MultiMintOpenEditionMinterQueryMsgExtension::AllMintInstances {},
        ),
    );
    assert_eq!(mint_instances.as_ref().unwrap()[0].1.minted_count, 1);
    // Query collector minting details
    let user_minting_details: Result<UserDetails, _> = app.wrap().query_wasm_smart(
        multi_minter_addr.clone(),
        &MultiMintOpenEditionMinterQueryMsg::Extension(
            MultiMintOpenEditionMinterQueryMsgExtension::UserMintingDetails {
                address: collector.to_string(),
                mint_instance_id: None,
            },
        ),
    );
    assert_eq!(user_minting_details.unwrap().public_mint_count, 1);

    // Try removing the mint_instance. Should fail as tokens are minted from this mint_instance
    let res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(multi_minter_addr.clone()),
            &MultiMintOpenEditionMinterExecuteMsg::RemoveMintInstance {
                mint_instance_id: 1,
            },
            &[coin(2000000, "uflix")],
        )
        .unwrap_err();
    let err = res
        .source()
        .unwrap()
        .downcast_ref::<MultiMintOpenEditionMinterContractError>()
        .unwrap();
    assert_eq!(
        err,
        &MultiMintOpenEditionMinterContractError::MintInstanceCantBeRemoved
    );

    // Add new mint_instance
    let token_details = TokenDetails {
        token_name: "MintInstance number 2".to_string(),
        description: Some("MintInstance number 2 description".to_string()),
        preview_uri: Some("MintInstance number 2 prev uri".to_string()),
        base_token_uri: "MintInstance number 2 base_token_uri".to_string(),
        transferable: true,
        royalty_ratio: Decimal::percent(10),
        extensible: true,
        nsfw: false,
        data: Some("MintInstance number 2 data".to_string()),
    };
    let config = Config {
        mint_price: coin(5_000_000, "uflix"),
        start_time: Timestamp::from_nanos(50_000_000),
        end_time: Some(Timestamp::from_nanos(100_000_000)),
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
    assert_eq!(active_mint_instance, 2);

    // Creator sends another mint_instance
    let token_details = TokenDetails {
        token_name: "MintInstance number 3".to_string(),
        description: Some("MintInstance number 3 description".to_string()),
        preview_uri: Some("MintInstance number 3 prev uri".to_string()),
        base_token_uri: "MintInstance number 3 base_token_uri".to_string(),
        transferable: true,
        royalty_ratio: Decimal::percent(10),
        extensible: true,
        nsfw: false,
        data: Some("MintInstance number 3 data".to_string()),
    };
    let config = Config {
        mint_price: coin(5_000_000, "uflix"),
        start_time: Timestamp::from_nanos(100_000_000),
        end_time: Some(Timestamp::from_nanos(150_000_000)),
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

    // Creator removes the last mint_instance
    let _res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(multi_minter_addr.clone()),
            &MultiMintOpenEditionMinterExecuteMsg::RemoveMintInstance {
                mint_instance_id: 3,
            },
            &[coin(2000000, "uflix")],
        )
        .unwrap();
    // Query all the mint_instances
    let mint_instances: Result<Vec<(u32, MintInstance)>, _> = app.wrap().query_wasm_smart(
        multi_minter_addr.clone(),
        &MultiMintOpenEditionMinterQueryMsg::Extension(
            MultiMintOpenEditionMinterQueryMsgExtension::AllMintInstances {},
        ),
    );
    assert_eq!(mint_instances.as_ref().unwrap().len(), 2);
    // Check active mint_instance id
    let active_mint_instance: u32 = app
        .wrap()
        .query_wasm_smart(
            multi_minter_addr.clone(),
            &MultiMintOpenEditionMinterQueryMsg::Extension(
                MultiMintOpenEditionMinterQueryMsgExtension::ActiveMintInstanceId {},
            ),
        )
        .unwrap();
    assert_eq!(active_mint_instance, 2);

    // Creator sends another mint_instance
    let token_details = TokenDetails {
        token_name: "MintInstance number 4".to_string(),
        description: Some("MintInstance number 4 description".to_string()),
        preview_uri: Some("MintInstance number 4 prev uri".to_string()),
        base_token_uri: "MintInstance number 4 base_token_uri".to_string(),
        transferable: true,
        royalty_ratio: Decimal::percent(10),
        extensible: true,
        nsfw: false,
        data: Some("MintInstance number 4 data".to_string()),
    };
    let config = Config {
        mint_price: coin(5_000_000, "uflix"),
        start_time: Timestamp::from_nanos(150_000_000),
        end_time: Some(Timestamp::from_nanos(200_000_000)),
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
    // Creator removed 3rd mint_instance so this mint_instances id should be 4
    let active_mint_instance: u32 = app
        .wrap()
        .query_wasm_smart(
            multi_minter_addr.clone(),
            &MultiMintOpenEditionMinterQueryMsg::Extension(
                MultiMintOpenEditionMinterQueryMsgExtension::ActiveMintInstanceId {},
            ),
        )
        .unwrap();
    assert_eq!(active_mint_instance, 4);

    // All mint_instances
    let mint_instances: Result<Vec<(u32, MintInstance)>, _> = app.wrap().query_wasm_smart(
        multi_minter_addr.clone(),
        &MultiMintOpenEditionMinterQueryMsg::Extension(
            MultiMintOpenEditionMinterQueryMsgExtension::AllMintInstances {},
        ),
    );
    // Extract mint_instance ids
    let mint_instance_ids: Vec<u32> = mint_instances
        .as_ref()
        .unwrap()
        .iter()
        .map(|(id, _)| *id)
        .collect();
    assert_eq!(mint_instance_ids, vec![1, 2, 4]);

    // Creator removes the last mint_instance
    // Active mint_instance id should be 2
    let _res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(multi_minter_addr.clone()),
            &MultiMintOpenEditionMinterExecuteMsg::RemoveMintInstance {
                mint_instance_id: active_mint_instance,
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
    assert_eq!(active_mint_instance, 2);
}
#[test]
fn remove_non_active_mint_instance() {
    let res = setup();
    let admin = res.test_accounts.admin;
    let creator = res.test_accounts.creator;
    let collector = res.test_accounts.collector;
    let open_edition_minter_factory_code_id = res.open_edition_minter_factory_code_id;
    let multi_mint_open_edition_minter_code_id = res.multi_mint_open_edition_minter_code_id;
    let mut app = res.app;
    // Instantiate the minter factory
    let open_edition_minter_factory_instantiate_msg =
        return_open_edition_minter_factory_inst_message(
            open_edition_minter_factory_code_id,
            Some(multi_mint_open_edition_minter_code_id),
        );

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
    let active_mint_instance: u32 = app
        .wrap()
        .query_wasm_smart(
            multi_minter_addr.clone(),
            &MultiMintOpenEditionMinterQueryMsg::Extension(
                MultiMintOpenEditionMinterQueryMsgExtension::ActiveMintInstanceId {},
            ),
        )
        .unwrap();
    assert_eq!(active_mint_instance, 0);
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
    // Now first mint_instance is active one
    // Lets add one more mint_instance
    let token_details = TokenDetails {
        token_name: "MintInstance number 2".to_string(),
        description: Some("MintInstance number 2 description".to_string()),
        preview_uri: Some("MintInstance number 2 prev uri".to_string()),
        base_token_uri: "MintInstance number 2 base_token_uri".to_string(),
        transferable: true,
        royalty_ratio: Decimal::percent(10),
        extensible: true,
        nsfw: false,
        data: Some("MintInstance number 2 data".to_string()),
    };
    let config = Config {
        mint_price: coin(5_000_000, "uflix"),
        start_time: Timestamp::from_nanos(50_000_000),
        end_time: Some(Timestamp::from_nanos(100_000_000)),
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
    // Now active mint_instance is 2
    // Both is removable as no tokens are minted

    // Lets add one more mint_instance and mint some tokens
    let token_details = TokenDetails {
        token_name: "MintInstance number 3".to_string(),
        description: Some("MintInstance number 3 description".to_string()),
        preview_uri: Some("MintInstance number 3 prev uri".to_string()),
        base_token_uri: "MintInstance number 3 base_token_uri".to_string(),
        transferable: true,
        royalty_ratio: Decimal::percent(10),
        extensible: true,
        nsfw: false,
        data: Some("MintInstance number 3 data".to_string()),
    };
    let config = Config {
        mint_price: coin(5_000_000, "uflix"),
        start_time: Timestamp::from_nanos(100_000_000),
        end_time: Some(Timestamp::from_nanos(150_000_000)),
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
    // Set time to public sale
    app.set_block(BlockInfo {
        time: Timestamp::from_nanos(110_000_000),
        height: 1,
        chain_id: "cosmos".to_string(),
    });

    // Mint token for the mint_instance
    app.execute_contract(
        collector.clone(),
        Addr::unchecked(multi_minter_addr.clone()),
        &MultiMintOpenEditionMinterExecuteMsg::Mint {
            mint_instance_id: Some(3),
        },
        &[coin(5_000_000, "uflix")],
    )
    .unwrap();

    // Now active mint_instance is 3 and it has tokens minted

    // Try removing mint_instance 3
    let res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(multi_minter_addr.clone()),
            &MultiMintOpenEditionMinterExecuteMsg::RemoveMintInstance {
                mint_instance_id: 3,
            },
            &[coin(2000000, "uflix")],
        )
        .unwrap_err();
    let err = res
        .source()
        .unwrap()
        .downcast_ref::<MultiMintOpenEditionMinterContractError>()
        .unwrap();
    assert_eq!(
        err,
        &MultiMintOpenEditionMinterContractError::MintInstanceCantBeRemoved {}
    );

    // Try removing mint_instance 1
    // This should pass as no tokens are minted from this mint_instance
    // But active mint_instance should not be changed
    let _res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(multi_minter_addr.clone()),
            &MultiMintOpenEditionMinterExecuteMsg::RemoveMintInstance {
                mint_instance_id: 1,
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
    assert_eq!(active_mint_instance, 3);

    // Query all the mint_instances
    let mint_instances: Result<Vec<(u32, MintInstance)>, _> = app.wrap().query_wasm_smart(
        multi_minter_addr.clone(),
        &MultiMintOpenEditionMinterQueryMsg::Extension(
            MultiMintOpenEditionMinterQueryMsgExtension::AllMintInstances {},
        ),
    );
    assert_eq!(mint_instances.as_ref().unwrap().len(), 2);
    // Extract mint_instance ids
    let mint_instance_ids: Vec<u32> = mint_instances
        .as_ref()
        .unwrap()
        .iter()
        .map(|(id, _)| *id)
        .collect();
    assert_eq!(mint_instance_ids, vec![2, 3]);

    // Try removing mint_instance 2
    // This should pass as no tokens are minted from this mint_instance
    // But active mint_instance should not be changed again
    let _res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(multi_minter_addr.clone()),
            &MultiMintOpenEditionMinterExecuteMsg::RemoveMintInstance {
                mint_instance_id: 2,
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
    assert_eq!(active_mint_instance, 3);

    // Query all the mint_instances
    let mint_instances: Result<Vec<(u32, MintInstance)>, _> = app.wrap().query_wasm_smart(
        multi_minter_addr.clone(),
        &MultiMintOpenEditionMinterQueryMsg::Extension(
            MultiMintOpenEditionMinterQueryMsgExtension::AllMintInstances {},
        ),
    );
    assert_eq!(mint_instances.as_ref().unwrap().len(), 1);
    // Extract mint_instance ids
    let mint_instance_ids: Vec<u32> = mint_instances
        .as_ref()
        .unwrap()
        .iter()
        .map(|(id, _)| *id)
        .collect();
    assert_eq!(mint_instance_ids, vec![3]);
}
#[test]
fn remove_first_mint_instance() {
    let res = setup();
    let admin = res.test_accounts.admin;
    let creator = res.test_accounts.creator;
    let collector = res.test_accounts.collector;
    let open_edition_minter_factory_code_id = res.open_edition_minter_factory_code_id;
    let multi_mint_open_edition_minter_code_id = res.multi_mint_open_edition_minter_code_id;
    let mut app = res.app;
    // Instantiate the minter factory
    let open_edition_minter_factory_instantiate_msg =
        return_open_edition_minter_factory_inst_message(
            open_edition_minter_factory_code_id,
            Some(multi_mint_open_edition_minter_code_id),
        );

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
    // MintInstance id 0 is not available to remove
    let res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(multi_minter_addr.clone()),
            &MultiMintOpenEditionMinterExecuteMsg::RemoveMintInstance {
                mint_instance_id: 0,
            },
            &[coin(2000000, "uflix")],
        )
        .unwrap_err();
    let err = res
        .source()
        .unwrap()
        .downcast_ref::<MultiMintOpenEditionMinterContractError>()
        .unwrap();
    assert_eq!(
        err,
        &MultiMintOpenEditionMinterContractError::NoMintInstanceAvailable {}
    );

    // Try removing the first mint_instance. Should fail as this mint_instance does not exist
    let res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(multi_minter_addr.clone()),
            &MultiMintOpenEditionMinterExecuteMsg::RemoveMintInstance {
                mint_instance_id: 1,
            },
            &[coin(2000000, "uflix")],
        )
        .unwrap_err();
    let err = res
        .source()
        .unwrap()
        .downcast_ref::<MultiMintOpenEditionMinterContractError>()
        .unwrap();
    assert_eq!(
        err,
        &MultiMintOpenEditionMinterContractError::InvalidMintInstanceId {}
    );

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

    // Query all the mint_instances
    let mint_instances: Result<Vec<(u32, MintInstance)>, _> = app.wrap().query_wasm_smart(
        multi_minter_addr.clone(),
        &MultiMintOpenEditionMinterQueryMsg::Extension(
            MultiMintOpenEditionMinterQueryMsgExtension::AllMintInstances {},
        ),
    );
    assert_eq!(mint_instances.as_ref().unwrap().len(), 1);

    // Creator removes the only mint_instance
    let _res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(multi_minter_addr.clone()),
            &MultiMintOpenEditionMinterExecuteMsg::RemoveMintInstance {
                mint_instance_id: active_mint_instance,
            },
            &[],
        )
        .unwrap();
    // Query all the mint_instances
    let res: Result<Vec<(u32, MintInstanceParams)>, _> = app.wrap().query_wasm_smart(
        multi_minter_addr.clone(),
        &MultiMintOpenEditionMinterQueryMsg::Extension(
            MultiMintOpenEditionMinterQueryMsgExtension::AllMintInstances {},
        ),
    );
    assert_eq!(res.as_ref().unwrap().len(), 0);

    // Check active mint_instance id
    let active_mint_instance: u32 = app
        .wrap()
        .query_wasm_smart(
            multi_minter_addr.clone(),
            &MultiMintOpenEditionMinterQueryMsg::Extension(
                MultiMintOpenEditionMinterQueryMsgExtension::ActiveMintInstanceId {},
            ),
        )
        .unwrap();
    assert_eq!(active_mint_instance, 0);
    // Try minting
    let res = app
        .execute_contract(
            collector.clone(),
            Addr::unchecked(multi_minter_addr.clone()),
            &MultiMintOpenEditionMinterExecuteMsg::Mint {
                mint_instance_id: None,
            },
            &[coin(5_000_000, "uflix")],
        )
        .unwrap_err();
    let err = res
        .source()
        .unwrap()
        .downcast_ref::<MultiMintOpenEditionMinterContractError>()
        .unwrap();
    assert_eq!(
        err,
        &MultiMintOpenEditionMinterContractError::NoMintInstanceAvailable {}
    );
}
#[test]
fn add_mint_instance() {
    let res = setup();
    let admin = res.test_accounts.admin;
    let creator = res.test_accounts.creator;
    let collector = res.test_accounts.collector;
    let open_edition_minter_factory_code_id = res.open_edition_minter_factory_code_id;
    let multi_mint_open_edition_minter_code_id = res.multi_mint_open_edition_minter_code_id;
    let mut app = res.app;
    // Instantiate the minter factory
    let open_edition_minter_factory_instantiate_msg =
        return_open_edition_minter_factory_inst_message(
            open_edition_minter_factory_code_id,
            Some(multi_mint_open_edition_minter_code_id),
        );

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

    let mint_instance = MintInstanceParams {
        token_details: TokenDetails {
            token_name: "MintInstance number 1".to_string(),
            description: Some("MintInstance number 1 description".to_string()),
            preview_uri: Some("MintInstance number 1 prev uri".to_string()),
            base_token_uri: "MintInstance number 1 base_token_uri".to_string(),
            transferable: true,
            royalty_ratio: Decimal::percent(10),
            extensible: true,
            nsfw: false,
            data: Some("MintInstance number 1 data".to_string()),
        },
        config: Config {
            mint_price: coin(5_000_000, "uflix"),
            start_time: Timestamp::from_nanos(10_000_000),
            end_time: Some(Timestamp::from_nanos(50_500_000)),
            per_address_limit: Some(100),
            whitelist_address: None,
            num_tokens: Some(100),
        },
    };
    // Non admin tries to add mint_instance
    let res = app
        .execute_contract(
            collector.clone(),
            Addr::unchecked(multi_minter_addr.clone()),
            &MultiMintOpenEditionMinterExecuteMsg::CreateMintInstance {
                config: mint_instance.config.clone(),
                token_details: mint_instance.token_details.clone(),
            },
            &[coin(2000000, "uflix")],
        )
        .unwrap_err();
    let err = res
        .source()
        .unwrap()
        .downcast_ref::<MultiMintOpenEditionMinterContractError>()
        .unwrap();
    assert_eq!(
        err,
        &MultiMintOpenEditionMinterContractError::Unauthorized {}
    );
    // Send too long token name
    let mut new_mint_instance = mint_instance.clone();
    new_mint_instance.token_details.token_name = "a".repeat(257);
    let res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(multi_minter_addr.clone()),
            &MultiMintOpenEditionMinterExecuteMsg::CreateMintInstance {
                config: new_mint_instance.config.clone(),
                token_details: new_mint_instance.token_details.clone(),
            },
            &[coin(2000000, "uflix")],
        )
        .unwrap_err();
    let err = res
        .source()
        .unwrap()
        .downcast_ref::<MultiMintOpenEditionMinterContractError>()
        .unwrap();
    assert_eq!(
        err,
        &MultiMintOpenEditionMinterContractError::TokenDetailsError(
            TokenDetailsError::TokenNameTooLong {}
        )
    );
    // Send too short token name
    let mut new_mint_instance = mint_instance.clone();
    new_mint_instance.token_details.token_name = " ".to_string();
    let res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(multi_minter_addr.clone()),
            &MultiMintOpenEditionMinterExecuteMsg::CreateMintInstance {
                config: new_mint_instance.config.clone(),
                token_details: new_mint_instance.token_details.clone(),
            },
            &[coin(2000000, "uflix")],
        )
        .unwrap_err();
    let err = res
        .source()
        .unwrap()
        .downcast_ref::<MultiMintOpenEditionMinterContractError>()
        .unwrap();
    assert_eq!(
        err,
        &MultiMintOpenEditionMinterContractError::TokenDetailsError(
            TokenDetailsError::TokenNameTooShort {}
        )
    );

    // Send too long token description
    let mut new_mint_instance = mint_instance.clone();
    new_mint_instance.token_details.description = Some("a".repeat(4097));
    let res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(multi_minter_addr.clone()),
            &MultiMintOpenEditionMinterExecuteMsg::CreateMintInstance {
                config: new_mint_instance.config.clone(),
                token_details: new_mint_instance.token_details.clone(),
            },
            &[coin(2000000, "uflix")],
        )
        .unwrap_err();
    let err = res
        .source()
        .unwrap()
        .downcast_ref::<MultiMintOpenEditionMinterContractError>()
        .unwrap();
    assert_eq!(
        err,
        &MultiMintOpenEditionMinterContractError::TokenDetailsError(
            TokenDetailsError::TokenDescriptionTooLong {}
        )
    );

    // Send too long token preview uri
    let mut new_mint_instance = mint_instance.clone();
    new_mint_instance.token_details.preview_uri = Some("a".repeat(257));
    let res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(multi_minter_addr.clone()),
            &MultiMintOpenEditionMinterExecuteMsg::CreateMintInstance {
                config: new_mint_instance.config.clone(),
                token_details: new_mint_instance.token_details.clone(),
            },
            &[coin(2000000, "uflix")],
        )
        .unwrap_err();
    let err = res
        .source()
        .unwrap()
        .downcast_ref::<MultiMintOpenEditionMinterContractError>()
        .unwrap();
    assert_eq!(
        err,
        &MultiMintOpenEditionMinterContractError::TokenDetailsError(
            TokenDetailsError::PreviewUriTooLong {}
        )
    );

    // Send too short token preview uri
    let mut new_mint_instance = mint_instance.clone();
    new_mint_instance.token_details.preview_uri = Some(" ".to_string());
    let res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(multi_minter_addr.clone()),
            &MultiMintOpenEditionMinterExecuteMsg::CreateMintInstance {
                config: new_mint_instance.config.clone(),
                token_details: new_mint_instance.token_details.clone(),
            },
            &[coin(2000000, "uflix")],
        )
        .unwrap_err();
    let err = res
        .source()
        .unwrap()
        .downcast_ref::<MultiMintOpenEditionMinterContractError>()
        .unwrap();
    assert_eq!(
        err,
        &MultiMintOpenEditionMinterContractError::TokenDetailsError(
            TokenDetailsError::PreviewUriTooShort {}
        )
    );

    // Send too long token data
    let mut new_mint_instance = mint_instance.clone();
    new_mint_instance.token_details.data = Some("a".repeat(4097));
    let res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(multi_minter_addr.clone()),
            &MultiMintOpenEditionMinterExecuteMsg::CreateMintInstance {
                config: new_mint_instance.config.clone(),
                token_details: new_mint_instance.token_details.clone(),
            },
            &[coin(2000000, "uflix")],
        )
        .unwrap_err();
    let err = res
        .source()
        .unwrap()
        .downcast_ref::<MultiMintOpenEditionMinterContractError>()
        .unwrap();
    assert_eq!(
        err,
        &MultiMintOpenEditionMinterContractError::TokenDetailsError(
            TokenDetailsError::DataTooLong {}
        )
    );

    // Send too long token base uri
    let mut new_mint_instance = mint_instance.clone();
    new_mint_instance.token_details.base_token_uri = "a".repeat(257);
    let res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(multi_minter_addr.clone()),
            &MultiMintOpenEditionMinterExecuteMsg::CreateMintInstance {
                config: new_mint_instance.config.clone(),
                token_details: new_mint_instance.token_details.clone(),
            },
            &[coin(2000000, "uflix")],
        )
        .unwrap_err();
    let err = res
        .source()
        .unwrap()
        .downcast_ref::<MultiMintOpenEditionMinterContractError>()
        .unwrap();
    assert_eq!(
        err,
        &MultiMintOpenEditionMinterContractError::TokenDetailsError(
            TokenDetailsError::BaseTokenUriTooLong {}
        )
    );

    // Send too short token base uri
    let mut new_mint_instance = mint_instance.clone();
    new_mint_instance.token_details.base_token_uri = " ".to_string();
    let res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(multi_minter_addr.clone()),
            &MultiMintOpenEditionMinterExecuteMsg::CreateMintInstance {
                config: new_mint_instance.config.clone(),
                token_details: new_mint_instance.token_details.clone(),
            },
            &[coin(2000000, "uflix")],
        )
        .unwrap_err();
    let err = res
        .source()
        .unwrap()
        .downcast_ref::<MultiMintOpenEditionMinterContractError>()
        .unwrap();
    assert_eq!(
        err,
        &MultiMintOpenEditionMinterContractError::TokenDetailsError(
            TokenDetailsError::BaseTokenUriTooShort {}
        )
    );

    // Send already active mint_instance
    let mut new_mint_instance = mint_instance.clone();
    new_mint_instance.config.start_time = Timestamp::from_nanos(0);

    let res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(multi_minter_addr.clone()),
            &MultiMintOpenEditionMinterExecuteMsg::CreateMintInstance {
                config: new_mint_instance.config.clone(),
                token_details: new_mint_instance.token_details.clone(),
            },
            &[coin(2000000, "uflix")],
        )
        .unwrap_err();
    let err = res
        .source()
        .unwrap()
        .downcast_ref::<MultiMintOpenEditionMinterContractError>()
        .unwrap();
    assert_eq!(
        err,
        &MultiMintOpenEditionMinterContractError::ConfigurationError(
            ConfigurationError::InvalidStartTime {}
        )
    );

    // Send end time before start time
    let mut new_mint_instance = mint_instance.clone();
    new_mint_instance.config.start_time = Timestamp::from_nanos(100);
    new_mint_instance.config.end_time = Some(Timestamp::from_nanos(50));

    let res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(multi_minter_addr.clone()),
            &MultiMintOpenEditionMinterExecuteMsg::CreateMintInstance {
                config: new_mint_instance.config.clone(),
                token_details: new_mint_instance.token_details.clone(),
            },
            &[coin(2000000, "uflix")],
        )
        .unwrap_err();
    let err = res
        .source()
        .unwrap()
        .downcast_ref::<MultiMintOpenEditionMinterContractError>()
        .unwrap();
    assert_eq!(
        err,
        &MultiMintOpenEditionMinterContractError::ConfigurationError(
            ConfigurationError::InvalidStartTime {}
        )
    );

    // Send zero per address limit
    let mut new_mint_instance = mint_instance.clone();
    new_mint_instance.config.per_address_limit = Some(0);

    let res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(multi_minter_addr.clone()),
            &MultiMintOpenEditionMinterExecuteMsg::CreateMintInstance {
                config: new_mint_instance.config.clone(),
                token_details: new_mint_instance.token_details.clone(),
            },
            &[coin(2000000, "uflix")],
        )
        .unwrap_err();
    let err = res
        .source()
        .unwrap()
        .downcast_ref::<MultiMintOpenEditionMinterContractError>()
        .unwrap();
    assert_eq!(
        err,
        &MultiMintOpenEditionMinterContractError::ConfigurationError(
            ConfigurationError::InvalidPerAddressLimit {}
        )
    );

    // Send zero number of tokens
    let mut new_mint_instance = mint_instance.clone();
    new_mint_instance.config.num_tokens = Some(0);

    let res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(multi_minter_addr.clone()),
            &MultiMintOpenEditionMinterExecuteMsg::CreateMintInstance {
                config: new_mint_instance.config.clone(),
                token_details: new_mint_instance.token_details.clone(),
            },
            &[coin(2000000, "uflix")],
        )
        .unwrap_err();
    let err = res
        .source()
        .unwrap()
        .downcast_ref::<MultiMintOpenEditionMinterContractError>()
        .unwrap();
    assert_eq!(
        err,
        &MultiMintOpenEditionMinterContractError::ConfigurationError(
            ConfigurationError::InvalidNumberOfTokens {}
        )
    );

    // Happy path
    let _res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(multi_minter_addr.clone()),
            &MultiMintOpenEditionMinterExecuteMsg::CreateMintInstance {
                config: mint_instance.config.clone(),
                token_details: mint_instance.token_details.clone(),
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
    // Query all the mint_instances
    let mint_instances: Result<Vec<(u32, MintInstance)>, _> = app.wrap().query_wasm_smart(
        multi_minter_addr.clone(),
        &MultiMintOpenEditionMinterQueryMsg::Extension(
            MultiMintOpenEditionMinterQueryMsgExtension::AllMintInstances {},
        ),
    );
    assert_eq!(mint_instances.as_ref().unwrap().len(), 1);
    // Check mint_instance id
    assert_eq!(mint_instances.as_ref().unwrap()[0].0, 1);
}

#[test]
fn update_mint_price() {
    let res = setup();
    let admin = res.test_accounts.admin;
    let creator = res.test_accounts.creator;
    let collector = res.test_accounts.collector;
    let open_edition_minter_factory_code_id = res.open_edition_minter_factory_code_id;
    let multi_mint_open_edition_minter_code_id = res.multi_mint_open_edition_minter_code_id;
    let mut app = res.app;
    // Instantiate the minter factory
    let open_edition_minter_factory_instantiate_msg =
        return_open_edition_minter_factory_inst_message(
            open_edition_minter_factory_code_id,
            Some(multi_mint_open_edition_minter_code_id),
        );

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
        auth_details: AuthDetails {
            admin: creator.clone(),
            payment_collector: creator.clone(),
        },
        init: Default::default(),
        token_details: None,
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
    // Try updating mint price without any mint_instance
    let res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(multi_minter_addr.clone()),
            &MultiMintOpenEditionMinterExecuteMsg::UpdateMintPrice {
                mint_price: coin(5_000_000, "uflix"),
                mint_instance_id: None,
            },
            &[coin(2000000, "uflix")],
        )
        .unwrap_err();
    let err = res
        .source()
        .unwrap()
        .downcast_ref::<MultiMintOpenEditionMinterContractError>()
        .unwrap();
    assert_eq!(
        err,
        &MultiMintOpenEditionMinterContractError::NoMintInstanceAvailable {}
    );

    let mint_instance = MintInstanceParams {
        token_details: TokenDetails {
            token_name: "MintInstance number 1".to_string(),
            description: Some("MintInstance number 1 description".to_string()),
            preview_uri: Some("MintInstance number 1 prev uri".to_string()),
            base_token_uri: "MintInstance number 1 base_token_uri".to_string(),
            transferable: true,
            royalty_ratio: Decimal::percent(10),
            extensible: true,
            nsfw: false,
            data: Some("MintInstance number 1 data".to_string()),
        },
        config: Config {
            mint_price: coin(5_000_000, "uflix"),
            start_time: Timestamp::from_nanos(10_000_000),
            end_time: Some(Timestamp::from_nanos(50_500_000)),
            per_address_limit: Some(100),
            whitelist_address: None,
            num_tokens: Some(100),
        },
    };
    // Add mint_instance
    let _res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(multi_minter_addr.clone()),
            &MultiMintOpenEditionMinterExecuteMsg::CreateMintInstance {
                config: mint_instance.config.clone(),
                token_details: mint_instance.token_details.clone(),
            },
            &[coin(2000000, "uflix")],
        )
        .unwrap();

    // Update mint price with non admin
    let res = app
        .execute_contract(
            collector.clone(),
            Addr::unchecked(multi_minter_addr.clone()),
            &MultiMintOpenEditionMinterExecuteMsg::UpdateMintPrice {
                mint_price: coin(5_000_000, "uflix"),
                mint_instance_id: None,
            },
            &[coin(2000000, "uflix")],
        )
        .unwrap_err();
    let err = res
        .source()
        .unwrap()
        .downcast_ref::<MultiMintOpenEditionMinterContractError>()
        .unwrap();
    assert_eq!(
        err,
        &MultiMintOpenEditionMinterContractError::Unauthorized {}
    );

    // Update mint price with invalid mint_instance id
    let res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(multi_minter_addr.clone()),
            &MultiMintOpenEditionMinterExecuteMsg::UpdateMintPrice {
                mint_price: coin(5_000_000, "uflix"),
                mint_instance_id: Some(2),
            },
            &[coin(2000000, "uflix")],
        )
        .unwrap_err();
    let err = res
        .source()
        .unwrap()
        .downcast_ref::<MultiMintOpenEditionMinterContractError>()
        .unwrap();
    assert_eq!(
        err,
        &MultiMintOpenEditionMinterContractError::InvalidMintInstanceId {}
    );

    // Update mint price
    let _res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(multi_minter_addr.clone()),
            &MultiMintOpenEditionMinterExecuteMsg::UpdateMintPrice {
                mint_price: coin(10_000_000, "uflix"),
                mint_instance_id: None,
            },
            &[coin(2000000, "uflix")],
        )
        .unwrap();
    // Query mint price
    let config: Config = app
        .wrap()
        .query_wasm_smart(
            multi_minter_addr.clone(),
            &MultiMintOpenEditionMinterQueryMsg::Extension(
                MultiMintOpenEditionMinterQueryMsgExtension::Config {
                    mint_instance_id: Some(1),
                },
            ),
        )
        .unwrap();
    assert_eq!(config.mint_price, coin(10_000_000, "uflix"));
}

#[test]
fn update_royalty_ratio() {
    let res = setup();
    let admin = res.test_accounts.admin;
    let creator = res.test_accounts.creator;
    let collector = res.test_accounts.collector;
    let open_edition_minter_factory_code_id = res.open_edition_minter_factory_code_id;
    let multi_mint_open_edition_minter_code_id = res.multi_mint_open_edition_minter_code_id;
    let mut app = res.app;
    // Instantiate the minter factory
    let open_edition_minter_factory_instantiate_msg =
        return_open_edition_minter_factory_inst_message(
            open_edition_minter_factory_code_id,
            Some(multi_mint_open_edition_minter_code_id),
        );

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
    // Try updating royalty ratio without any mint_instance
    let res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(multi_minter_addr.clone()),
            &MultiMintOpenEditionMinterExecuteMsg::UpdateRoyaltyRatio {
                ratio: Decimal::percent(10).to_string(),
                mint_instance_id: None,
            },
            &[coin(2000000, "uflix")],
        )
        .unwrap_err();
    let err = res
        .source()
        .unwrap()
        .downcast_ref::<MultiMintOpenEditionMinterContractError>()
        .unwrap();
    assert_eq!(
        err,
        &MultiMintOpenEditionMinterContractError::NoMintInstanceAvailable {}
    );

    let mint_instance = MintInstanceParams {
        token_details: TokenDetails {
            token_name: "MintInstance number 1".to_string(),
            description: Some("MintInstance number 1 description".to_string()),
            preview_uri: Some("MintInstance number 1 prev uri".to_string()),
            base_token_uri: "MintInstance number 1 base_token_uri".to_string(),
            transferable: true,
            royalty_ratio: Decimal::percent(10),
            extensible: true,
            nsfw: false,
            data: Some("MintInstance number 1 data".to_string()),
        },
        config: Config {
            mint_price: coin(5_000_000, "uflix"),
            start_time: Timestamp::from_nanos(10_000_000),
            end_time: Some(Timestamp::from_nanos(50_500_000)),
            per_address_limit: Some(100),
            whitelist_address: None,
            num_tokens: Some(100),
        },
    };
    // Add mint_instance
    let _res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(multi_minter_addr.clone()),
            &MultiMintOpenEditionMinterExecuteMsg::CreateMintInstance {
                config: mint_instance.config.clone(),
                token_details: mint_instance.token_details.clone(),
            },
            &[coin(2000000, "uflix")],
        )
        .unwrap();

    // Update royalty ratio with non admin
    let res = app
        .execute_contract(
            collector.clone(),
            Addr::unchecked(multi_minter_addr.clone()),
            &MultiMintOpenEditionMinterExecuteMsg::UpdateRoyaltyRatio {
                ratio: Decimal::percent(10).to_string(),
                mint_instance_id: None,
            },
            &[coin(2000000, "uflix")],
        )
        .unwrap_err();
    let err = res
        .source()
        .unwrap()
        .downcast_ref::<MultiMintOpenEditionMinterContractError>()
        .unwrap();
    assert_eq!(
        err,
        &MultiMintOpenEditionMinterContractError::Unauthorized {}
    );

    // Update royalty ratio with invalid mint_instance id
    let res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(multi_minter_addr.clone()),
            &MultiMintOpenEditionMinterExecuteMsg::UpdateRoyaltyRatio {
                ratio: Decimal::percent(10).to_string(),
                mint_instance_id: Some(2),
            },
            &[coin(2000000, "uflix")],
        )
        .unwrap_err();
    let err = res
        .source()
        .unwrap()
        .downcast_ref::<MultiMintOpenEditionMinterContractError>()
        .unwrap();
    assert_eq!(
        err,
        &MultiMintOpenEditionMinterContractError::InvalidMintInstanceId {}
    );
    // Send invalid ratio
    let res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(multi_minter_addr.clone()),
            &MultiMintOpenEditionMinterExecuteMsg::UpdateRoyaltyRatio {
                ratio: "One".to_string(),
                mint_instance_id: None,
            },
            &[coin(2000000, "uflix")],
        )
        .unwrap_err();
    let err = res
        .source()
        .unwrap()
        .downcast_ref::<MultiMintOpenEditionMinterContractError>()
        .unwrap();
    assert_eq!(
        err,
        &MultiMintOpenEditionMinterContractError::Std(StdError::generic_err("Error parsing whole")),
    );

    // Send ratio more than 100%
    let res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(multi_minter_addr.clone()),
            &MultiMintOpenEditionMinterExecuteMsg::UpdateRoyaltyRatio {
                ratio: Decimal::percent(101).to_string(),
                mint_instance_id: None,
            },
            &[coin(2000000, "uflix")],
        )
        .unwrap_err();
    let err = res
        .source()
        .unwrap()
        .downcast_ref::<MultiMintOpenEditionMinterContractError>()
        .unwrap();
    assert_eq!(
        err,
        &MultiMintOpenEditionMinterContractError::TokenDetailsError(
            TokenDetailsError::InvalidRoyaltyRatio {}
        )
    );

    // Update royalty ratio
    let _res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(multi_minter_addr.clone()),
            &MultiMintOpenEditionMinterExecuteMsg::UpdateRoyaltyRatio {
                ratio: Decimal::percent(20).to_string(),
                mint_instance_id: None,
            },
            &[coin(2000000, "uflix")],
        )
        .unwrap();
}
