#![cfg(test)]
use cosmwasm_std::{coin, Addr, BlockInfo, Timestamp};
use cosmwasm_std::{Decimal, Uint128};
use cw_multi_test::Executor;
use minter_types::msg::QueryMsg as CommonMinterQueryMsg;
use minter_types::types::TokenDetails;
use minter_types::types::{CollectionDetails, Config, UserDetails};
use omniflix_multi_mint_open_edition_minter::error::ContractError as MultiMintOpenEditionMinterContractError;
use omniflix_multi_mint_open_edition_minter::msg::ExecuteMsg as MultiMintOpenEditionMinterExecuteMsg;
use omniflix_multi_mint_open_edition_minter::msg::QueryMsgExtension as MultiMintOpenEditionMinterQueryMsgExtension;
use omniflix_open_edition_minter_factory::msg::{
    ExecuteMsg as OpenEditionMinterFactoryExecuteMsg, MultiMinterCreateMsg,
    MultiMinterInitExtention,
};

type MultiMintOpenEditionMinterQueryMsg =
    CommonMinterQueryMsg<MultiMintOpenEditionMinterQueryMsgExtension>;

use crate::helpers::mock_messages::factory_mock_messages::return_open_edition_minter_factory_inst_message;
use crate::helpers::utils::{get_contract_address_from_res, query_onft_collection};

use crate::helpers::setup::setup;

#[test]
fn test_multi_mint_oem_public_minting() {
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
    // Try minting without an active drop
    let res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(multi_minter_addr.clone()),
            &MultiMintOpenEditionMinterExecuteMsg::Mint { drop_id: None },
            &[coin(2000000, "uflix")],
        )
        .unwrap_err();
    let error = res.source().unwrap();
    let error = error
        .downcast_ref::<MultiMintOpenEditionMinterContractError>()
        .unwrap();
    assert_eq!(
        error,
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

    // Try minting before the start time
    let res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(multi_minter_addr.clone()),
            &MultiMintOpenEditionMinterExecuteMsg::Mint { drop_id: Some(1) },
            &[coin(2000000, "uflix")],
        )
        .unwrap_err();
    let error = res.source().unwrap();
    let error = error
        .downcast_ref::<MultiMintOpenEditionMinterContractError>()
        .unwrap();
    assert_eq!(
        error,
        &MultiMintOpenEditionMinterContractError::MintingNotStarted {
            start_time: Timestamp::from_nanos(10_000_000),
            current_time: Timestamp::from_nanos(1_000)
        }
    );
    // Try MintAdmin should work before the start time
    let _res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(multi_minter_addr.clone()),
            &MultiMintOpenEditionMinterExecuteMsg::MintAdmin {
                drop_id: Some(1),
                recipient: creator.clone().into_string(),
            },
            &[],
        )
        .unwrap();
    // Query collection
    let onft_collection = query_onft_collection(app.storage(), multi_minter_addr.clone());
    assert_eq!(onft_collection.onfts.len(), 1);
    let onft = &onft_collection.onfts[0];
    assert_eq!(onft.id, 1.to_string());
    assert_eq!(onft.owner, creator.to_string());
    assert_eq!(
        onft.metadata.as_ref().unwrap().name,
        "Drop number 1 #1".to_string()
    );
    // Query user details
    let user_details: UserDetails = app
        .wrap()
        .query_wasm_smart(
            &multi_minter_addr,
            &MultiMintOpenEditionMinterQueryMsg::UserMintingDetails {
                address: creator.clone().into_string(),
            },
        )
        .unwrap();
    assert_eq!(user_details.public_mint_count, 0);
    assert_eq!(user_details.total_minted_count, 1);
    assert_eq!(user_details.minted_tokens.len(), 1);

    // Query minted count
    let minted_count: u32 = app
        .wrap()
        .query_wasm_smart(
            &multi_minter_addr,
            &MultiMintOpenEditionMinterQueryMsg::TotalMintedCount {},
        )
        .unwrap();
    assert_eq!(minted_count, 1);

    // Query minted count in drop
    let minted_count: u32 = app
        .wrap()
        .query_wasm_smart(
            &multi_minter_addr,
            &MultiMintOpenEditionMinterQueryMsg::Extension(
                MultiMintOpenEditionMinterQueryMsgExtension::TokensMintedInDrop {
                    drop_id: Some(1),
                },
            ),
        )
        .unwrap();
    assert_eq!(minted_count, 1);

    // Try minting after the end time
    app.set_block(BlockInfo {
        height: 1,
        time: Timestamp::from_nanos(60_000_000),
        chain_id: "test".to_string(),
    });
    let res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(multi_minter_addr.clone()),
            &MultiMintOpenEditionMinterExecuteMsg::Mint { drop_id: Some(1) },
            &[coin(2000000, "uflix")],
        )
        .unwrap_err();
    let error = res.source().unwrap();
    let error = error
        .downcast_ref::<MultiMintOpenEditionMinterContractError>()
        .unwrap();
    assert_eq!(
        error,
        &MultiMintOpenEditionMinterContractError::PublicMintingEnded {}
    );

    // Try minting with insufficient funds
    app.set_block(BlockInfo {
        height: 1,
        time: Timestamp::from_nanos(20_000_000),
        chain_id: "test".to_string(),
    });
    let res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(multi_minter_addr.clone()),
            &MultiMintOpenEditionMinterExecuteMsg::Mint { drop_id: Some(1) },
            &[coin(2000000, "uflix")],
        )
        .unwrap_err();
    let error = res.source().unwrap();
    let error = error
        .downcast_ref::<MultiMintOpenEditionMinterContractError>()
        .unwrap();
    assert_eq!(
        error,
        &MultiMintOpenEditionMinterContractError::IncorrectPaymentAmount {
            expected: Uint128::from(5000000u128),
            sent: Uint128::from(2000000u128)
        }
    );
    // Check creator balance before mint
    let creator_balance_before_mint: Uint128 = app
        .wrap()
        .query_balance(creator.to_string(), "uflix")
        .unwrap()
        .amount;

    // Mint with correct funds
    let _res = app
        .execute_contract(
            collector.clone(),
            Addr::unchecked(multi_minter_addr.clone()),
            &MultiMintOpenEditionMinterExecuteMsg::Mint { drop_id: Some(1) },
            &[coin(5000000, "uflix")],
        )
        .unwrap();
    // Check creator balance after mint
    let creator_balance_after_mint: Uint128 = app
        .wrap()
        .query_balance(creator.to_string(), "uflix")
        .unwrap()
        .amount;
    assert_eq!(
        creator_balance_after_mint - creator_balance_before_mint,
        Uint128::from(5000000u128)
    );
    // Query collection
    let onft_collection = query_onft_collection(app.storage(), multi_minter_addr.clone());
    assert_eq!(onft_collection.onfts.len(), 2);
    let onft = &onft_collection.onfts[1];
    assert_eq!(onft.id, 2.to_string());
    assert_eq!(onft.owner, collector.to_string());
    assert_eq!(
        onft.metadata.as_ref().unwrap().name,
        "Drop number 1 #2".to_string()
    );

    // Query user details
    let user_details: UserDetails = app
        .wrap()
        .query_wasm_smart(
            &multi_minter_addr,
            &MultiMintOpenEditionMinterQueryMsg::UserMintingDetails {
                address: collector.clone().into_string(),
            },
        )
        .unwrap();
    assert_eq!(user_details.public_mint_count, 1);
    assert_eq!(user_details.total_minted_count, 1);
    assert_eq!(user_details.minted_tokens.len(), 1);
    assert_eq!(user_details.minted_tokens[0].token_id, 2.to_string());

    // Mint every remaining token
    for i in 3..=100 {
        let _res = app
            .execute_contract(
                collector.clone(),
                Addr::unchecked(multi_minter_addr.clone()),
                &MultiMintOpenEditionMinterExecuteMsg::Mint { drop_id: Some(1) },
                &[coin(5000000, "uflix")],
            )
            .unwrap();
        // Query collection
        let onft_collection = query_onft_collection(app.storage(), multi_minter_addr.clone());
        assert_eq!(onft_collection.onfts.len(), i as usize);
        let onft = &onft_collection.onfts[i as usize - 1];
        assert_eq!(onft.id, i.to_string());
        assert_eq!(onft.owner, collector.to_string());
        assert_eq!(
            onft.metadata.as_ref().unwrap().name,
            format!("Drop number 1 #{}", i)
        );
    }

    // Try minting after drop is fully minted
    let res = app
        .execute_contract(
            collector.clone(),
            Addr::unchecked(multi_minter_addr.clone()),
            &MultiMintOpenEditionMinterExecuteMsg::Mint { drop_id: Some(1) },
            &[coin(5000000, "uflix")],
        )
        .unwrap_err();
    let error = res.source().unwrap();
    let error = error
        .downcast_ref::<MultiMintOpenEditionMinterContractError>()
        .unwrap();
    assert_eq!(
        error,
        &MultiMintOpenEditionMinterContractError::NoTokensLeftToMint {}
    );

    // Try MintAdmin after drop is fully minted
    let res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(multi_minter_addr.clone()),
            &MultiMintOpenEditionMinterExecuteMsg::MintAdmin {
                drop_id: Some(1),
                recipient: creator.clone().into_string(),
            },
            &[],
        )
        .unwrap_err();
    let error = res.source().unwrap();
    let error = error
        .downcast_ref::<MultiMintOpenEditionMinterContractError>()
        .unwrap();
    assert_eq!(
        error,
        &MultiMintOpenEditionMinterContractError::NoTokensLeftToMint {}
    );
}
