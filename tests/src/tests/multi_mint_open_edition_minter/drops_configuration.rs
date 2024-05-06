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

use omniflix_multi_mint_open_edition_minter::drop::{Drop, DropParams};
use omniflix_open_edition_minter_factory::msg::{
    ExecuteMsg as OpenEditionMinterFactoryExecuteMsg, MultiMinterCreateMsg,
};

type MultiMintOpenEditionMinterQueryMsg =
    CommonMinterQueryMsg<MultiMintOpenEditionMinterQueryMsgExtension>;

use crate::helpers::mock_messages::factory_mock_messages::return_open_edition_minter_factory_inst_message;
use crate::helpers::utils::get_contract_address_from_res;

use crate::helpers::setup::setup;

#[test]
fn remove_drop() {
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
    let active_drop: u32 = app
        .wrap()
        .query_wasm_smart(
            multi_minter_addr.clone(),
            &MultiMintOpenEditionMinterQueryMsg::Extension(
                MultiMintOpenEditionMinterQueryMsgExtension::ActiveDropId {},
            ),
        )
        .unwrap();
    assert_eq!(active_drop, 0);
    // Create first drop
    let token_details = TokenDetails {
        token_name: "Drop number 1".to_string(),
        description: Some("Drop number 1 description".to_string()),
        preview_uri: Some("Drop number 1 prev uri".to_string()),
        base_token_uri: "Drop number 1 base_token_uri".to_string(),
        transferable: true,
        royalty_ratio: Decimal::percent(10),
        extensible: true,
        nsfw: false,
        data: Some("Drop number 1 data".to_string()),
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
            &MultiMintOpenEditionMinterExecuteMsg::NewDrop {
                config,
                token_details,
            },
            &[coin(2000000, "uflix")],
        )
        .unwrap();

    let active_drop: u32 = app
        .wrap()
        .query_wasm_smart(
            multi_minter_addr.clone(),
            &MultiMintOpenEditionMinterQueryMsg::Extension(
                MultiMintOpenEditionMinterQueryMsgExtension::ActiveDropId {},
            ),
        )
        .unwrap();
    assert_eq!(active_drop, 1);

    // Query all the drops
    let drops: Result<Vec<(u32, Drop)>, _> = app.wrap().query_wasm_smart(
        multi_minter_addr.clone(),
        &MultiMintOpenEditionMinterQueryMsg::Extension(
            MultiMintOpenEditionMinterQueryMsgExtension::AllDrops {},
        ),
    );
    assert_eq!(drops.as_ref().unwrap().len(), 1);
    // Check drop id
    assert_eq!(drops.as_ref().unwrap()[0].0, 1);

    // Set time to public sale
    app.set_block(BlockInfo {
        time: Timestamp::from_nanos(20_000_000),
        height: 1,
        chain_id: "cosmos".to_string(),
    });

    // Mint token for the drop
    app.execute_contract(
        collector.clone(),
        Addr::unchecked(multi_minter_addr.clone()),
        &MultiMintOpenEditionMinterExecuteMsg::Mint { drop_id: None },
        &[coin(5_000_000, "uflix")],
    )
    .unwrap();
    // Query all the drops
    let drops: Result<Vec<(u32, Drop)>, _> = app.wrap().query_wasm_smart(
        multi_minter_addr.clone(),
        &MultiMintOpenEditionMinterQueryMsg::Extension(
            MultiMintOpenEditionMinterQueryMsgExtension::AllDrops {},
        ),
    );
    assert_eq!(drops.as_ref().unwrap()[0].1.minted_count, 1);
    // Query collector minting details
    let user_minting_details: Result<UserDetails, _> = app.wrap().query_wasm_smart(
        multi_minter_addr.clone(),
        &MultiMintOpenEditionMinterQueryMsg::Extension(
            MultiMintOpenEditionMinterQueryMsgExtension::UserMintingDetails {
                address: collector.to_string(),
                drop_id: None,
            },
        ),
    );
    assert_eq!(user_minting_details.unwrap().public_mint_count, 1);

    // Try removing the drop. Should fail as tokens are minted from this drop
    let res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(multi_minter_addr.clone()),
            &MultiMintOpenEditionMinterExecuteMsg::RemoveDrop { drop_id: 1 },
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
        &MultiMintOpenEditionMinterContractError::DropCantBeRemoved
    );

    // Add new drop
    let token_details = TokenDetails {
        token_name: "Drop number 2".to_string(),
        description: Some("Drop number 2 description".to_string()),
        preview_uri: Some("Drop number 2 prev uri".to_string()),
        base_token_uri: "Drop number 2 base_token_uri".to_string(),
        transferable: true,
        royalty_ratio: Decimal::percent(10),
        extensible: true,
        nsfw: false,
        data: Some("Drop number 2 data".to_string()),
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
            &MultiMintOpenEditionMinterExecuteMsg::NewDrop {
                config,
                token_details,
            },
            &[coin(2000000, "uflix")],
        )
        .unwrap();

    let active_drop: u32 = app
        .wrap()
        .query_wasm_smart(
            multi_minter_addr.clone(),
            &MultiMintOpenEditionMinterQueryMsg::Extension(
                MultiMintOpenEditionMinterQueryMsgExtension::ActiveDropId {},
            ),
        )
        .unwrap();
    assert_eq!(active_drop, 2);

    // Creator sends another drop
    let token_details = TokenDetails {
        token_name: "Drop number 3".to_string(),
        description: Some("Drop number 3 description".to_string()),
        preview_uri: Some("Drop number 3 prev uri".to_string()),
        base_token_uri: "Drop number 3 base_token_uri".to_string(),
        transferable: true,
        royalty_ratio: Decimal::percent(10),
        extensible: true,
        nsfw: false,
        data: Some("Drop number 3 data".to_string()),
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
            &MultiMintOpenEditionMinterExecuteMsg::NewDrop {
                config,
                token_details,
            },
            &[coin(2000000, "uflix")],
        )
        .unwrap();

    // Creator removes the last drop
    let _res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(multi_minter_addr.clone()),
            &MultiMintOpenEditionMinterExecuteMsg::RemoveDrop { drop_id: 3 },
            &[coin(2000000, "uflix")],
        )
        .unwrap();
    // Query all the drops
    let drops: Result<Vec<(u32, Drop)>, _> = app.wrap().query_wasm_smart(
        multi_minter_addr.clone(),
        &MultiMintOpenEditionMinterQueryMsg::Extension(
            MultiMintOpenEditionMinterQueryMsgExtension::AllDrops {},
        ),
    );
    assert_eq!(drops.as_ref().unwrap().len(), 2);
    // Check active drop id
    let active_drop: u32 = app
        .wrap()
        .query_wasm_smart(
            multi_minter_addr.clone(),
            &MultiMintOpenEditionMinterQueryMsg::Extension(
                MultiMintOpenEditionMinterQueryMsgExtension::ActiveDropId {},
            ),
        )
        .unwrap();
    assert_eq!(active_drop, 2);

    // Creator sends another drop
    let token_details = TokenDetails {
        token_name: "Drop number 4".to_string(),
        description: Some("Drop number 4 description".to_string()),
        preview_uri: Some("Drop number 4 prev uri".to_string()),
        base_token_uri: "Drop number 4 base_token_uri".to_string(),
        transferable: true,
        royalty_ratio: Decimal::percent(10),
        extensible: true,
        nsfw: false,
        data: Some("Drop number 4 data".to_string()),
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
            &MultiMintOpenEditionMinterExecuteMsg::NewDrop {
                config,
                token_details,
            },
            &[coin(2000000, "uflix")],
        )
        .unwrap();
    // Creator removed 3rd drop so this drops id should be 4
    let active_drop: u32 = app
        .wrap()
        .query_wasm_smart(
            multi_minter_addr.clone(),
            &MultiMintOpenEditionMinterQueryMsg::Extension(
                MultiMintOpenEditionMinterQueryMsgExtension::ActiveDropId {},
            ),
        )
        .unwrap();
    assert_eq!(active_drop, 4);

    // All drops
    let drops: Result<Vec<(u32, Drop)>, _> = app.wrap().query_wasm_smart(
        multi_minter_addr.clone(),
        &MultiMintOpenEditionMinterQueryMsg::Extension(
            MultiMintOpenEditionMinterQueryMsgExtension::AllDrops {},
        ),
    );
    // Extract drop ids
    let drop_ids: Vec<u32> = drops.as_ref().unwrap().iter().map(|(id, _)| *id).collect();
    assert_eq!(drop_ids, vec![1, 2, 4]);

    // Creator removes the last drop
    // Active drop id should be 2
    let _res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(multi_minter_addr.clone()),
            &MultiMintOpenEditionMinterExecuteMsg::RemoveDrop {
                drop_id: active_drop,
            },
            &[coin(2000000, "uflix")],
        )
        .unwrap();
    let active_drop: u32 = app
        .wrap()
        .query_wasm_smart(
            multi_minter_addr.clone(),
            &MultiMintOpenEditionMinterQueryMsg::Extension(
                MultiMintOpenEditionMinterQueryMsgExtension::ActiveDropId {},
            ),
        )
        .unwrap();
    assert_eq!(active_drop, 2);
}
#[test]
fn remove_non_active_drop() {
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
    let active_drop: u32 = app
        .wrap()
        .query_wasm_smart(
            multi_minter_addr.clone(),
            &MultiMintOpenEditionMinterQueryMsg::Extension(
                MultiMintOpenEditionMinterQueryMsgExtension::ActiveDropId {},
            ),
        )
        .unwrap();
    assert_eq!(active_drop, 0);
    // Create first drop
    let token_details = TokenDetails {
        token_name: "Drop number 1".to_string(),
        description: Some("Drop number 1 description".to_string()),
        preview_uri: Some("Drop number 1 prev uri".to_string()),
        base_token_uri: "Drop number 1 base_token_uri".to_string(),
        transferable: true,
        royalty_ratio: Decimal::percent(10),
        extensible: true,
        nsfw: false,
        data: Some("Drop number 1 data".to_string()),
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
            &MultiMintOpenEditionMinterExecuteMsg::NewDrop {
                config,
                token_details,
            },
            &[coin(2000000, "uflix")],
        )
        .unwrap();
    // Now first drop is active one
    // Lets add one more drop
    let token_details = TokenDetails {
        token_name: "Drop number 2".to_string(),
        description: Some("Drop number 2 description".to_string()),
        preview_uri: Some("Drop number 2 prev uri".to_string()),
        base_token_uri: "Drop number 2 base_token_uri".to_string(),
        transferable: true,
        royalty_ratio: Decimal::percent(10),
        extensible: true,
        nsfw: false,
        data: Some("Drop number 2 data".to_string()),
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
            &MultiMintOpenEditionMinterExecuteMsg::NewDrop {
                config,
                token_details,
            },
            &[coin(2000000, "uflix")],
        )
        .unwrap();
    // Now active drop is 2
    // Both is removable as no tokens are minted

    // Lets add one more drop and mint some tokens
    let token_details = TokenDetails {
        token_name: "Drop number 3".to_string(),
        description: Some("Drop number 3 description".to_string()),
        preview_uri: Some("Drop number 3 prev uri".to_string()),
        base_token_uri: "Drop number 3 base_token_uri".to_string(),
        transferable: true,
        royalty_ratio: Decimal::percent(10),
        extensible: true,
        nsfw: false,
        data: Some("Drop number 3 data".to_string()),
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
            &MultiMintOpenEditionMinterExecuteMsg::NewDrop {
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

    // Mint token for the drop
    app.execute_contract(
        collector.clone(),
        Addr::unchecked(multi_minter_addr.clone()),
        &MultiMintOpenEditionMinterExecuteMsg::Mint { drop_id: Some(3) },
        &[coin(5_000_000, "uflix")],
    )
    .unwrap();

    // Now active drop is 3 and it has tokens minted

    // Try removing drop 3
    let res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(multi_minter_addr.clone()),
            &MultiMintOpenEditionMinterExecuteMsg::RemoveDrop { drop_id: 3 },
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
        &MultiMintOpenEditionMinterContractError::DropCantBeRemoved {}
    );

    // Try removing drop 1
    // This should pass as no tokens are minted from this drop
    // But active drop should not be changed
    let _res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(multi_minter_addr.clone()),
            &MultiMintOpenEditionMinterExecuteMsg::RemoveDrop { drop_id: 1 },
            &[coin(2000000, "uflix")],
        )
        .unwrap();
    let active_drop: u32 = app
        .wrap()
        .query_wasm_smart(
            multi_minter_addr.clone(),
            &MultiMintOpenEditionMinterQueryMsg::Extension(
                MultiMintOpenEditionMinterQueryMsgExtension::ActiveDropId {},
            ),
        )
        .unwrap();
    assert_eq!(active_drop, 3);

    // Query all the drops
    let drops: Result<Vec<(u32, Drop)>, _> = app.wrap().query_wasm_smart(
        multi_minter_addr.clone(),
        &MultiMintOpenEditionMinterQueryMsg::Extension(
            MultiMintOpenEditionMinterQueryMsgExtension::AllDrops {},
        ),
    );
    assert_eq!(drops.as_ref().unwrap().len(), 2);
    // Extract drop ids
    let drop_ids: Vec<u32> = drops.as_ref().unwrap().iter().map(|(id, _)| *id).collect();
    assert_eq!(drop_ids, vec![2, 3]);

    // Try removing drop 2
    // This should pass as no tokens are minted from this drop
    // But active drop should not be changed again
    let _res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(multi_minter_addr.clone()),
            &MultiMintOpenEditionMinterExecuteMsg::RemoveDrop { drop_id: 2 },
            &[coin(2000000, "uflix")],
        )
        .unwrap();
    let active_drop: u32 = app
        .wrap()
        .query_wasm_smart(
            multi_minter_addr.clone(),
            &MultiMintOpenEditionMinterQueryMsg::Extension(
                MultiMintOpenEditionMinterQueryMsgExtension::ActiveDropId {},
            ),
        )
        .unwrap();
    assert_eq!(active_drop, 3);

    // Query all the drops
    let drops: Result<Vec<(u32, Drop)>, _> = app.wrap().query_wasm_smart(
        multi_minter_addr.clone(),
        &MultiMintOpenEditionMinterQueryMsg::Extension(
            MultiMintOpenEditionMinterQueryMsgExtension::AllDrops {},
        ),
    );
    assert_eq!(drops.as_ref().unwrap().len(), 1);
    // Extract drop ids
    let drop_ids: Vec<u32> = drops.as_ref().unwrap().iter().map(|(id, _)| *id).collect();
    assert_eq!(drop_ids, vec![3]);
}
#[test]
fn remove_first_drop() {
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
    // Drop id 0 is not available to remove
    let res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(multi_minter_addr.clone()),
            &MultiMintOpenEditionMinterExecuteMsg::RemoveDrop { drop_id: 0 },
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
        &MultiMintOpenEditionMinterContractError::NoDropAvailable {}
    );

    // Try removing the first drop. Should fail as this drop does not exist
    let res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(multi_minter_addr.clone()),
            &MultiMintOpenEditionMinterExecuteMsg::RemoveDrop { drop_id: 1 },
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
        &MultiMintOpenEditionMinterContractError::InvalidDropId {}
    );

    // Create first drop
    let token_details = TokenDetails {
        token_name: "Drop number 1".to_string(),
        description: Some("Drop number 1 description".to_string()),
        preview_uri: Some("Drop number 1 prev uri".to_string()),
        base_token_uri: "Drop number 1 base_token_uri".to_string(),
        transferable: true,
        royalty_ratio: Decimal::percent(10),
        extensible: true,
        nsfw: false,
        data: Some("Drop number 1 data".to_string()),
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
            &MultiMintOpenEditionMinterExecuteMsg::NewDrop {
                config,
                token_details,
            },
            &[coin(2000000, "uflix")],
        )
        .unwrap();

    let active_drop: u32 = app
        .wrap()
        .query_wasm_smart(
            multi_minter_addr.clone(),
            &MultiMintOpenEditionMinterQueryMsg::Extension(
                MultiMintOpenEditionMinterQueryMsgExtension::ActiveDropId {},
            ),
        )
        .unwrap();
    assert_eq!(active_drop, 1);

    // Query all the drops
    let drops: Result<Vec<(u32, Drop)>, _> = app.wrap().query_wasm_smart(
        multi_minter_addr.clone(),
        &MultiMintOpenEditionMinterQueryMsg::Extension(
            MultiMintOpenEditionMinterQueryMsgExtension::AllDrops {},
        ),
    );
    assert_eq!(drops.as_ref().unwrap().len(), 1);

    // Creator removes the only drop
    let _res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(multi_minter_addr.clone()),
            &MultiMintOpenEditionMinterExecuteMsg::RemoveDrop {
                drop_id: active_drop,
            },
            &[],
        )
        .unwrap();
    // Query all the drops
    let res: Result<Vec<(u32, DropParams)>, _> = app.wrap().query_wasm_smart(
        multi_minter_addr.clone(),
        &MultiMintOpenEditionMinterQueryMsg::Extension(
            MultiMintOpenEditionMinterQueryMsgExtension::AllDrops {},
        ),
    );
    assert_eq!(res.as_ref().unwrap().len(), 0);

    // Check active drop id
    let active_drop: u32 = app
        .wrap()
        .query_wasm_smart(
            multi_minter_addr.clone(),
            &MultiMintOpenEditionMinterQueryMsg::Extension(
                MultiMintOpenEditionMinterQueryMsgExtension::ActiveDropId {},
            ),
        )
        .unwrap();
    assert_eq!(active_drop, 0);
    // Try minting
    let res = app
        .execute_contract(
            collector.clone(),
            Addr::unchecked(multi_minter_addr.clone()),
            &MultiMintOpenEditionMinterExecuteMsg::Mint { drop_id: None },
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
        &MultiMintOpenEditionMinterContractError::NoDropAvailable {}
    );
}
#[test]
fn add_drop() {
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

    let drop = DropParams {
        token_details: TokenDetails {
            token_name: "Drop number 1".to_string(),
            description: Some("Drop number 1 description".to_string()),
            preview_uri: Some("Drop number 1 prev uri".to_string()),
            base_token_uri: "Drop number 1 base_token_uri".to_string(),
            transferable: true,
            royalty_ratio: Decimal::percent(10),
            extensible: true,
            nsfw: false,
            data: Some("Drop number 1 data".to_string()),
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
    // Non admin tries to add drop
    let res = app
        .execute_contract(
            collector.clone(),
            Addr::unchecked(multi_minter_addr.clone()),
            &MultiMintOpenEditionMinterExecuteMsg::NewDrop {
                config: drop.config.clone(),
                token_details: drop.token_details.clone(),
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
    let mut new_drop = drop.clone();
    new_drop.token_details.token_name = "a".repeat(257);
    let res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(multi_minter_addr.clone()),
            &MultiMintOpenEditionMinterExecuteMsg::NewDrop {
                config: new_drop.config.clone(),
                token_details: new_drop.token_details.clone(),
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
    let mut new_drop = drop.clone();
    new_drop.token_details.token_name = " ".to_string();
    let res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(multi_minter_addr.clone()),
            &MultiMintOpenEditionMinterExecuteMsg::NewDrop {
                config: new_drop.config.clone(),
                token_details: new_drop.token_details.clone(),
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
    let mut new_drop = drop.clone();
    new_drop.token_details.description = Some("a".repeat(4097));
    let res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(multi_minter_addr.clone()),
            &MultiMintOpenEditionMinterExecuteMsg::NewDrop {
                config: new_drop.config.clone(),
                token_details: new_drop.token_details.clone(),
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
    let mut new_drop = drop.clone();
    new_drop.token_details.preview_uri = Some("a".repeat(257));
    let res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(multi_minter_addr.clone()),
            &MultiMintOpenEditionMinterExecuteMsg::NewDrop {
                config: new_drop.config.clone(),
                token_details: new_drop.token_details.clone(),
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
    let mut new_drop = drop.clone();
    new_drop.token_details.preview_uri = Some(" ".to_string());
    let res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(multi_minter_addr.clone()),
            &MultiMintOpenEditionMinterExecuteMsg::NewDrop {
                config: new_drop.config.clone(),
                token_details: new_drop.token_details.clone(),
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
    let mut new_drop = drop.clone();
    new_drop.token_details.data = Some("a".repeat(4097));
    let res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(multi_minter_addr.clone()),
            &MultiMintOpenEditionMinterExecuteMsg::NewDrop {
                config: new_drop.config.clone(),
                token_details: new_drop.token_details.clone(),
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
    let mut new_drop = drop.clone();
    new_drop.token_details.base_token_uri = "a".repeat(257);
    let res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(multi_minter_addr.clone()),
            &MultiMintOpenEditionMinterExecuteMsg::NewDrop {
                config: new_drop.config.clone(),
                token_details: new_drop.token_details.clone(),
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
    let mut new_drop = drop.clone();
    new_drop.token_details.base_token_uri = " ".to_string();
    let res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(multi_minter_addr.clone()),
            &MultiMintOpenEditionMinterExecuteMsg::NewDrop {
                config: new_drop.config.clone(),
                token_details: new_drop.token_details.clone(),
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

    // Send already active drop
    let mut new_drop = drop.clone();
    new_drop.config.start_time = Timestamp::from_nanos(0);

    let res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(multi_minter_addr.clone()),
            &MultiMintOpenEditionMinterExecuteMsg::NewDrop {
                config: new_drop.config.clone(),
                token_details: new_drop.token_details.clone(),
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
    let mut new_drop = drop.clone();
    new_drop.config.start_time = Timestamp::from_nanos(100);
    new_drop.config.end_time = Some(Timestamp::from_nanos(50));

    let res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(multi_minter_addr.clone()),
            &MultiMintOpenEditionMinterExecuteMsg::NewDrop {
                config: new_drop.config.clone(),
                token_details: new_drop.token_details.clone(),
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
    let mut new_drop = drop.clone();
    new_drop.config.per_address_limit = Some(0);

    let res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(multi_minter_addr.clone()),
            &MultiMintOpenEditionMinterExecuteMsg::NewDrop {
                config: new_drop.config.clone(),
                token_details: new_drop.token_details.clone(),
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
    let mut new_drop = drop.clone();
    new_drop.config.num_tokens = Some(0);

    let res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(multi_minter_addr.clone()),
            &MultiMintOpenEditionMinterExecuteMsg::NewDrop {
                config: new_drop.config.clone(),
                token_details: new_drop.token_details.clone(),
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
            &MultiMintOpenEditionMinterExecuteMsg::NewDrop {
                config: drop.config.clone(),
                token_details: drop.token_details.clone(),
            },
            &[coin(2000000, "uflix")],
        )
        .unwrap();
    let active_drop: u32 = app
        .wrap()
        .query_wasm_smart(
            multi_minter_addr.clone(),
            &MultiMintOpenEditionMinterQueryMsg::Extension(
                MultiMintOpenEditionMinterQueryMsgExtension::ActiveDropId {},
            ),
        )
        .unwrap();
    assert_eq!(active_drop, 1);
    // Query all the drops
    let drops: Result<Vec<(u32, Drop)>, _> = app.wrap().query_wasm_smart(
        multi_minter_addr.clone(),
        &MultiMintOpenEditionMinterQueryMsg::Extension(
            MultiMintOpenEditionMinterQueryMsgExtension::AllDrops {},
        ),
    );
    assert_eq!(drops.as_ref().unwrap().len(), 1);
    // Check drop id
    assert_eq!(drops.as_ref().unwrap()[0].0, 1);
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
    // Try updating mint price without any drop
    let res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(multi_minter_addr.clone()),
            &MultiMintOpenEditionMinterExecuteMsg::UpdateMintPrice {
                mint_price: coin(5_000_000, "uflix"),
                drop_id: None,
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
        &MultiMintOpenEditionMinterContractError::NoDropAvailable {}
    );

    let drop = DropParams {
        token_details: TokenDetails {
            token_name: "Drop number 1".to_string(),
            description: Some("Drop number 1 description".to_string()),
            preview_uri: Some("Drop number 1 prev uri".to_string()),
            base_token_uri: "Drop number 1 base_token_uri".to_string(),
            transferable: true,
            royalty_ratio: Decimal::percent(10),
            extensible: true,
            nsfw: false,
            data: Some("Drop number 1 data".to_string()),
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
    // Add drop
    let _res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(multi_minter_addr.clone()),
            &MultiMintOpenEditionMinterExecuteMsg::NewDrop {
                config: drop.config.clone(),
                token_details: drop.token_details.clone(),
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
                drop_id: None,
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

    // Update mint price with invalid drop id
    let res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(multi_minter_addr.clone()),
            &MultiMintOpenEditionMinterExecuteMsg::UpdateMintPrice {
                mint_price: coin(5_000_000, "uflix"),
                drop_id: Some(2),
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
        &MultiMintOpenEditionMinterContractError::InvalidDropId {}
    );

    // Update mint price
    let _res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(multi_minter_addr.clone()),
            &MultiMintOpenEditionMinterExecuteMsg::UpdateMintPrice {
                mint_price: coin(10_000_000, "uflix"),
                drop_id: None,
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
                MultiMintOpenEditionMinterQueryMsgExtension::Config { drop_id: Some(1) },
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
    // Try updating royalty ratio without any drop
    let res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(multi_minter_addr.clone()),
            &MultiMintOpenEditionMinterExecuteMsg::UpdateRoyaltyRatio {
                ratio: Decimal::percent(10).to_string(),
                drop_id: None,
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
        &MultiMintOpenEditionMinterContractError::NoDropAvailable {}
    );

    let drop = DropParams {
        token_details: TokenDetails {
            token_name: "Drop number 1".to_string(),
            description: Some("Drop number 1 description".to_string()),
            preview_uri: Some("Drop number 1 prev uri".to_string()),
            base_token_uri: "Drop number 1 base_token_uri".to_string(),
            transferable: true,
            royalty_ratio: Decimal::percent(10),
            extensible: true,
            nsfw: false,
            data: Some("Drop number 1 data".to_string()),
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
    // Add drop
    let _res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(multi_minter_addr.clone()),
            &MultiMintOpenEditionMinterExecuteMsg::NewDrop {
                config: drop.config.clone(),
                token_details: drop.token_details.clone(),
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
                drop_id: None,
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

    // Update royalty ratio with invalid drop id
    let res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(multi_minter_addr.clone()),
            &MultiMintOpenEditionMinterExecuteMsg::UpdateRoyaltyRatio {
                ratio: Decimal::percent(10).to_string(),
                drop_id: Some(2),
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
        &MultiMintOpenEditionMinterContractError::InvalidDropId {}
    );
    // Send invalid ratio
    let res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(multi_minter_addr.clone()),
            &MultiMintOpenEditionMinterExecuteMsg::UpdateRoyaltyRatio {
                ratio: "One".to_string(),
                drop_id: None,
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
                drop_id: None,
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
                drop_id: None,
            },
            &[coin(2000000, "uflix")],
        )
        .unwrap();
}
