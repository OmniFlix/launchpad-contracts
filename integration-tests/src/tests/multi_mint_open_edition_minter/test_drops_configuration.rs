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
use omniflix_open_edition_minter_factory::msg::{
    ExecuteMsg as OpenEditionMinterFactoryExecuteMsg, MultiMinterCreateMsg,
    MultiMinterInitExtention,
};

type MultiMintOpenEditionMinterQueryMsg =
    CommonMinterQueryMsg<MultiMintOpenEditionMinterQueryMsgExtension>;

use crate::helpers::utils::{
    get_contract_address_from_res, return_open_edition_minter_factory_inst_message,
};

use crate::helpers::setup::setup;

#[test]
fn test_remove_drop() {
    let (
        mut app,
        test_addresses,
        _minter_factory_code_id,
        _minter_code_id,
        _round_whitelist_factory_code_id,
        _round_whitelist_code_id,
        open_edition_minter_factory_code_id,
        _open_edition_minter_code_id,
        multi_minter_code_id,
    ) = setup();
    let admin = test_addresses.admin;
    let creator = test_addresses.creator;
    let collector = test_addresses.collector;
    // Instantiate the minter factory
    let open_edition_minter_factory_instantiate_msg =
        return_open_edition_minter_factory_inst_message(
            open_edition_minter_factory_code_id,
            multi_minter_code_id,
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
    let init = MultiMinterInitExtention {
        admin: creator.to_string(),
        payment_collector: Some(creator.to_string()),
    };

    let multi_minter_inst_msg = MultiMinterCreateMsg {
        collection_details,
        init,
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
            &MultiMintOpenEditionMinterExecuteMsg::RemoveDrop {},
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
            &MultiMintOpenEditionMinterExecuteMsg::RemoveDrop {},
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
            &MultiMintOpenEditionMinterExecuteMsg::RemoveDrop {},
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
fn test_remove_first_drop() {
    let (
        mut app,
        test_addresses,
        _minter_factory_code_id,
        _minter_code_id,
        _round_whitelist_factory_code_id,
        _round_whitelist_code_id,
        open_edition_minter_factory_code_id,
        _open_edition_minter_code_id,
        multi_minter_code_id,
    ) = setup();
    let admin = test_addresses.admin;
    let creator = test_addresses.creator;
    let collector = test_addresses.collector;
    // Instantiate the minter factory
    let open_edition_minter_factory_instantiate_msg =
        return_open_edition_minter_factory_inst_message(
            open_edition_minter_factory_code_id,
            multi_minter_code_id,
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
    let init = MultiMinterInitExtention {
        admin: creator.to_string(),
        payment_collector: Some(creator.to_string()),
    };

    let multi_minter_inst_msg = MultiMinterCreateMsg {
        collection_details,
        init,
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
    // First try removing the drop without creating any drop
    let res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(multi_minter_addr.clone()),
            &MultiMintOpenEditionMinterExecuteMsg::RemoveDrop {},
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
            &MultiMintOpenEditionMinterExecuteMsg::RemoveDrop {},
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
