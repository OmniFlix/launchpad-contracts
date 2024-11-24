#![cfg(test)]
use cosmwasm_std::Decimal;
use cosmwasm_std::{coin, Addr, BlockInfo, Timestamp};
use cw_multi_test::Executor;
use cw_utils::PaymentError;
use minter_types::collection_details::CollectionDetails;
use minter_types::config::Config;
use minter_types::msg::QueryMsg as CommonMinterQueryMsg;
use minter_types::token_details::TokenDetails;
use minter_types::types::{AuthDetails, UserDetails};
use omniflix_multi_mint_open_edition_minter::error::ContractError as MultiMintOpenEditionMinterContractError;
use omniflix_multi_mint_open_edition_minter::msg::ExecuteMsg as MultiMintOpenEditionMinterExecuteMsg;
use omniflix_multi_mint_open_edition_minter::msg::QueryMsgExtension as MultiMintOpenEditionMinterQueryMsgExtension;
use omniflix_open_edition_minter_factory::msg::{
    ExecuteMsg as OpenEditionMinterFactoryExecuteMsg, MultiMinterCreateMsg,
};

use omniflix_round_whitelist::error::ContractError as RoundWhitelistError;

use omniflix_round_whitelist_factory::msg::ExecuteMsg as RoundWhitelistFactoryExecuteMsg;
use whitelist_types::CreateWhitelistMsg;

type MultiMintOpenEditionMinterQueryMsg =
    CommonMinterQueryMsg<MultiMintOpenEditionMinterQueryMsgExtension>;

use crate::helpers::mock_messages::factory_mock_messages::{
    return_open_edition_minter_factory_inst_message, return_round_whitelist_factory_inst_message,
};
use crate::helpers::mock_messages::whitelist_mock_messages::return_round_configs;
use crate::helpers::utils::{get_contract_address_from_res, query_onft_collection};

use crate::helpers::setup::setup;

#[test]
fn multi_mint_oem_private_minting() {
    let res = setup();
    let admin = res.test_accounts.admin;
    let creator = res.test_accounts.creator;
    let collector = res.test_accounts.collector;
    let open_edition_minter_factory_code_id = res.open_edition_minter_factory_code_id;
    let multi_mint_open_edition_minter_code_id = res.multi_mint_open_edition_minter_code_id;
    let round_whitelist_factory_code_id = res.round_whitelist_factory_code_id;
    let round_whitelist_code_id = res.round_whitelist_code_id;
    let mut app = res.app;
    // Instantiate the minter factory
    let open_edition_minter_factory_instantiate_msg =
        return_open_edition_minter_factory_inst_message(
            open_edition_minter_factory_code_id,
            Some(multi_mint_open_edition_minter_code_id),
        );
    let round_whitelist_factory_instantiate_msg =
        return_round_whitelist_factory_inst_message(round_whitelist_code_id);

    let round_whitelist_factory_address = app
        .instantiate_contract(
            round_whitelist_factory_code_id,
            admin.clone(),
            &round_whitelist_factory_instantiate_msg,
            &[],
            "Round Whitelist Factory",
            None,
        )
        .unwrap();

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
    // Create a mm oem minter
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

    let minter_addr = get_contract_address_from_res(res);

    // Get rounds
    let rounds = return_round_configs();
    let round_1_start_at = rounds[0].round.start_time;
    let round_1_price = rounds[0].round.mint_price.clone();
    let round_2_price = rounds[1].round.mint_price.clone();
    let round_whitelist_inst_msg = CreateWhitelistMsg {
        admin: admin.to_string(),
        rounds: rounds.clone(),
    };

    // Create a whitelist
    let res = app
        .execute_contract(
            creator.clone(),
            round_whitelist_factory_address.clone(),
            &RoundWhitelistFactoryExecuteMsg::CreateWhitelist {
                msg: round_whitelist_inst_msg,
            },
            &[coin(1000000, "uflix")],
        )
        .unwrap();
    let round_whitelist_addr = get_contract_address_from_res(res);

    // Create a mint_instance
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
        whitelist_address: Some(Addr::unchecked(round_whitelist_addr.clone())),
        num_tokens: Some(100),
    };

    // Create a mint_instance
    let _res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(minter_addr.clone()),
            &MultiMintOpenEditionMinterExecuteMsg::CreateMintInstance {
                token_details: token_details.clone(),
                config: config.clone(),
            },
            &[coin(1000000, "uflix")],
        )
        .unwrap();

    // Private minting havent started yet
    // Try to mint
    let mint_msg = MultiMintOpenEditionMinterExecuteMsg::Mint { mint_instance_id: None };
    let res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(minter_addr.clone()),
            &mint_msg,
            &[round_1_price.clone()],
        )
        .unwrap_err();
    let error = res.source().unwrap();
    let error = error
        .downcast_ref::<MultiMintOpenEditionMinterContractError>()
        .unwrap();
    assert_eq!(
        error,
        &MultiMintOpenEditionMinterContractError::WhitelistNotActive {}
    );

    // Set time to first round
    let block = BlockInfo {
        time: round_1_start_at,
        height: 0,
        chain_id: "test".to_string(),
    };
    app.set_block(block);

    // Try to mint creator is not whitelisted for the first round
    let mint_msg = MultiMintOpenEditionMinterExecuteMsg::Mint { mint_instance_id: None };
    let res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(minter_addr.clone()),
            &mint_msg,
            &[round_1_price.clone()],
        )
        .unwrap_err();
    let error = res.source().unwrap();
    let error = error
        .downcast_ref::<MultiMintOpenEditionMinterContractError>()
        .unwrap();
    assert_eq!(
        error,
        &MultiMintOpenEditionMinterContractError::AddressNotWhitelisted {}
    );

    // Collector can mint but first send wrong payment
    let mint_msg = MultiMintOpenEditionMinterExecuteMsg::Mint { mint_instance_id: None };
    let res = app
        .execute_contract(
            collector.clone(),
            Addr::unchecked(minter_addr.clone()),
            &mint_msg,
            &[round_2_price.clone()],
        )
        .unwrap_err();
    let error = res.source().unwrap();
    let error = error
        .downcast_ref::<MultiMintOpenEditionMinterContractError>()
        .unwrap();
    assert_eq!(
        error,
        &MultiMintOpenEditionMinterContractError::PaymentError(PaymentError::ExtraDenom(
            round_2_price.denom.clone()
        ))
    );

    // Collector can mint
    let mint_msg = MultiMintOpenEditionMinterExecuteMsg::Mint { mint_instance_id: None };
    let _res = app
        .execute_contract(
            collector.clone(),
            Addr::unchecked(minter_addr.clone()),
            &mint_msg,
            &[round_1_price.clone()],
        )
        .unwrap();

    // Query the collection
    let collection = query_onft_collection(app.storage(), minter_addr.clone());
    assert_eq!(collection.onfts.len(), 1);
    let onft = collection.onfts.first().unwrap();
    assert_eq!(onft.owner, collector);
    assert_eq!(onft.id, 1.to_string());

    // Query the contract
    let total_minted_count: u32 = app
        .wrap()
        .query_wasm_smart(
            minter_addr.clone(),
            &MultiMintOpenEditionMinterQueryMsg::TotalMintedCount {},
        )
        .unwrap();
    assert_eq!(total_minted_count, 1);

    let user_minting_details: UserDetails = app
        .wrap()
        .query_wasm_smart(
            minter_addr.clone(),
            &MultiMintOpenEditionMinterQueryMsg::UserMintingDetails {
                address: collector.clone().into_string(),
            },
        )
        .unwrap();
    assert_eq!(user_minting_details.total_minted_count, 1);
    assert_eq!(user_minting_details.public_mint_count, 0);
    assert_eq!(user_minting_details.minted_tokens.len(), 1);
    assert_eq!(
        user_minting_details.minted_tokens.first().unwrap().token_id,
        1.to_string()
    );

    let minted_count_in_mint_instance: u32 = app
        .wrap()
        .query_wasm_smart(
            minter_addr.clone(),
            &MultiMintOpenEditionMinterQueryMsg::Extension(
                MultiMintOpenEditionMinterQueryMsgExtension::TokensMintedInMintInstance {
                    mint_instance_id: Some(1),
                },
            ),
        )
        .unwrap();
    assert_eq!(minted_count_in_mint_instance, 1);

    let tokens_remaining_in_mint_instance: u32 = app
        .wrap()
        .query_wasm_smart(
            minter_addr.clone(),
            &MultiMintOpenEditionMinterQueryMsg::Extension(
                MultiMintOpenEditionMinterQueryMsgExtension::TokensRemainingInMintInstance {
                    mint_instance_id: None,
                },
            ),
        )
        .unwrap();
    assert_eq!(tokens_remaining_in_mint_instance, 99);

    // Try minting again with the same collector
    // Should fail
    let mint_msg = MultiMintOpenEditionMinterExecuteMsg::Mint { mint_instance_id: None };
    let res = app
        .execute_contract(
            collector.clone(),
            Addr::unchecked(minter_addr.clone()),
            &mint_msg,
            &[round_1_price.clone()],
        )
        .unwrap_err();
    let error = res.source().unwrap().source().unwrap();
    let error = error.downcast_ref::<RoundWhitelistError>().unwrap();
    assert_eq!(error, &RoundWhitelistError::RoundReachedMintLimit {});
}
