#![cfg(test)]
use cosmwasm_std::{coin, coins, Addr, BlockInfo, Timestamp, Uint128};
use cosmwasm_std::{Decimal, StdError};
use cw_multi_test::Executor;
use minter_types::collection_details::CollectionDetails;
use minter_types::config::Config;
use minter_types::msg::QueryMsg as CommonMinterQueryMsg;
use minter_types::token_details::TokenDetails;
use minter_types::types::AuthDetails;
use omniflix_minter::msg::ExecuteMsg as MinterExecuteMsg;
use omniflix_multi_mint_open_edition_minter::error::ContractError as MultiMintOpenEditionMinterContractError;
use omniflix_multi_mint_open_edition_minter::msg::ExecuteMsg as MultiMintOpenEditionMinterExecuteMsg;
use omniflix_multi_mint_open_edition_minter::msg::QueryMsgExtension as MultiMintOpenEditionMinterQueryMsgExtension;

use omniflix_multi_mint_open_edition_minter::mint_instance::MintInstance;
use omniflix_open_edition_minter_factory::msg::{
    ExecuteMsg as OpenEditionMinterFactoryExecuteMsg, MultiMinterCreateMsg,
};
use omniflix_round_whitelist::msg::ExecuteMsg as RoundWhitelistExecuteMsg;
use whitelist_types::RoundConfig;
use whitelist_types::{CreateWhitelistMsg, Round};

type MultiMintOpenEditionMinterQueryMsg =
    CommonMinterQueryMsg<MultiMintOpenEditionMinterQueryMsgExtension>;

use crate::helpers::mock_messages::factory_mock_messages::{
    return_open_edition_minter_factory_inst_message, return_round_whitelist_factory_inst_message,
};
use crate::helpers::utils::{get_contract_address_from_res, mint_to_address};

use crate::{helpers::setup::setup, helpers::utils::query_onft_collection};

#[test]
fn test_scenario_2() {
    // Scenario 2:
    // 1. Creator creates a new round whitelist contract with 3 rounds
    // Rounds: 1, 2, 3
    // Round_1:
    // Round{
    //     start_time: 1_000_000,
    //     end_time: 2_000_000,
    //     round_per_address_limit: 100,
    //     addresses: ["collector"+ 1...25],
    //     round_price : 1_000_000 uflix
    // }
    // Round_2:
    // Round{
    //     start_time: 2_000_000,
    //     end_time: 3_000_000,
    //     round_per_address_limit: 100,
    //     addresses: ["collector"+ 1 ... 25],
    //     round_price : 2_000_000 ibc_atom
    // }
    // Round_3:
    // Round{
    //     start_time: 3_000_000,
    //     end_time: 4_000_000,
    //     round_per_address_limit: 1,
    //     addresses: ["collector"+ 26 ... 50],
    //     round_price : 3_000_000 ibc_atom
    // }
    // Creator creates first MultiMint contract with round whitelist
    // Num tokens: 100
    // Public mint start time: 10_000_000
    // Public mint end time: 50_500_000
    // Public per address limit: 1
    // Public mint price: 5_000_000 uflix
    // Private mint ends and only 25 NFTs are minted
    // Creator decides to add 1 more round
    // Round_4:
    // Round{
    //     start_time: 4_500_000,
    //     end_time: 8_000_000,
    //     round_per_address_limit: 100,
    //     addresses: ["collector"+ 1 ... 100],
    //     round_price : 200_000 uflix
    // }
    // 25 more NFTs are minted
    // Creator now decides to  create one more whitelist contract with 1 round
    // Round_1:
    // Round{
    //     start_time: 8_000_000,
    //     end_time: 11_000_000,
    //     round_per_address_limit: 100,
    //     addresses: ["collector"+ 1 ... 100],
    //     round_price : 200_000 uflix
    // }
    // 25 more NFTs are minted
    // Only 25 NFTs are remaining
    // On public mint start time, 10 NFTs are minted
    // Creator decides to lower the public price to 1_000_000 uflix
    // Creator makes a mistake and sets price to 1 uflix
    // Creator quickly pauses the contract
    // Creator fixes the price to 1_000_000 uflix
    // Creator unpauses the contract but decides to add one more pauser
    // Creator unpauses the contract with set pausers execution
    // Creator also decides to change royalty ratio to 20%
    // Creator also changes payment collector and admin addresses
    // Now Creator 2 decides create a new mint_instance
    let res = setup();
    let admin = res.test_accounts.admin;
    let creator = res.test_accounts.creator;
    let collector = res.test_accounts.collector;
    let multi_minter_code_id = res.multi_mint_open_edition_minter_code_id;
    let open_edition_minter_factory_code_id = res.open_edition_minter_factory_code_id;
    let round_whitelist_factory_code_id = res.round_whitelist_factory_code_id;
    let round_whitelist_code_id = res.round_whitelist_code_id;
    let mut app = res.app;

    // Instantiate the minter factory
    let open_edition_minter_factory_instantiate_msg =
        return_open_edition_minter_factory_inst_message(
            open_edition_minter_factory_code_id,
            Some(multi_minter_code_id),
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
    let rounds: Vec<Round> = [
        Round {
            start_time: Timestamp::from_nanos(1_000_000),
            end_time: Timestamp::from_nanos(2_000_000),
            round_per_address_limit: 100,
            mint_price: coin(1_000_000, "uflix"),
        },
        Round {
            start_time: Timestamp::from_nanos(2_000_000),
            end_time: Timestamp::from_nanos(3_000_000),
            round_per_address_limit: 100,
            mint_price: coin(2_000_000, "ibc_atom"),
        },
        Round {
            start_time: Timestamp::from_nanos(3_000_000),
            end_time: Timestamp::from_nanos(4_000_000),
            round_per_address_limit: 1,
            mint_price: coin(3_000_000, "ibc_atom"),
        },
    ]
    .to_vec();

    let round_1_members = (1..=25)
        .map(|i| format!("collector{}", i))
        .collect::<Vec<String>>();

    let round_2_members = (1..=25)
        .map(|i| format!("collector{}", i))
        .collect::<Vec<String>>();

    let round_3_members = (26..=50)
        .map(|i| format!("collector{}", i))
        .collect::<Vec<String>>();

    let round_1_config = RoundConfig {
        round: rounds[0].clone(),
        members: round_1_members.clone(),
    };
    let round_2_config = RoundConfig {
        round: rounds[1].clone(),
        members: round_2_members.clone(),
    };
    let round_3_config = RoundConfig {
        round: rounds[2].clone(),
        members: round_3_members.clone(),
    };

    let round_configs = vec![round_1_config, round_2_config, round_3_config];

    let round_whitelist_inst_msg = CreateWhitelistMsg {
        admin: admin.to_string(),
        rounds: round_configs.clone(),
    };
    let create_round_whitelist_msg =
        omniflix_round_whitelist_factory::msg::ExecuteMsg::CreateWhitelist {
            msg: round_whitelist_inst_msg,
        };
    let res = app
        .execute_contract(
            creator.clone(),
            round_whitelist_factory_addr.clone(),
            &create_round_whitelist_msg,
            &[coin(1000000, "uflix")],
        )
        .unwrap();
    let round_whitelist_addr = get_contract_address_from_res(res);
    let collection_details = CollectionDetails {
        collection_name: "Multi mint test".to_string(),
        description: Some("COLLECTION DESCRIPTION".to_string()),
        preview_uri: Some("Preview uri of COLLECTION".to_string()),
        schema: Some("Some schema for mint_instance 1".to_string()),
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

    // Query all mint_instances
    let res: Result<Vec<(u32, MintInstance)>, _> = app.wrap().query_wasm_smart(
        multi_minter_addr.clone(),
        &MultiMintOpenEditionMinterQueryMsg::Extension(
            MultiMintOpenEditionMinterQueryMsgExtension::AllMintInstances {},
        ),
    );
    assert_eq!(res.unwrap().len(), 0);
    let current_mint_instance_number: u32 = app
        .wrap()
        .query_wasm_smart(
            multi_minter_addr.clone(),
            &MultiMintOpenEditionMinterQueryMsg::Extension(
                MultiMintOpenEditionMinterQueryMsgExtension::ActiveMintInstanceId {},
            ),
        )
        .unwrap();
    assert_eq!(current_mint_instance_number, 0);

    let tokens_remaining_error: Result<u32, _> = app.wrap().query_wasm_smart(
        multi_minter_addr.clone(),
        &MultiMintOpenEditionMinterQueryMsg::Extension(
            MultiMintOpenEditionMinterQueryMsgExtension::TokensRemainingInMintInstance { mint_instance_id: Some(1) },
        ),
    );
    assert_eq!(
        tokens_remaining_error.unwrap_err(),
        StdError::generic_err("Querier contract error: Generic error: Invalid mint_instance id")
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
        whitelist_address: Some(Addr::unchecked(round_whitelist_addr.clone())),
        num_tokens: Some(100),
    };

    let _res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(multi_minter_addr.clone()),
            &MultiMintOpenEditionMinterExecuteMsg::NewMintInstance {
                token_details,
                config,
            },
            &[coin(2000000, "uflix")],
        )
        .unwrap();

    // Set time to first round
    app.set_block(BlockInfo {
        chain_id: "test_1".to_string(),
        height: 1_000,
        time: Timestamp::from_nanos(1_000_000 + 1),
    });
    // Mint nft for 10 collectors
    // Payment collector is creator
    // Check balance of creator
    let before_balance = app.wrap().query_balance(creator.clone(), "uflix").unwrap();
    for i in 1..=10 {
        let collector = Addr::unchecked(format!("collector{}", i));
        mint_to_address(&mut app, collector.to_string(), coins(1000000, "uflix"));
        let _res = app
            .execute_contract(
                collector.clone(),
                Addr::unchecked(multi_minter_addr.clone()),
                &MinterExecuteMsg::Mint {},
                &[coin(1000000, "uflix")],
            )
            .unwrap();
    }
    let after_balance = app.wrap().query_balance(creator.clone(), "uflix").unwrap();
    assert_eq!(
        after_balance.amount - before_balance.amount,
        Uint128::from(10000000u128)
    );
    let collection = query_onft_collection(app.storage(), multi_minter_addr.clone());
    assert_eq!(collection.onfts.len(), 10);
    // Set time to second round
    app.set_block(BlockInfo {
        chain_id: "test_1".to_string(),
        height: 1_000,
        time: Timestamp::from_nanos(2_000_000 + 1),
    });
    // Mint nft for 10 collectors

    for i in 1..=10 {
        let collector = Addr::unchecked(format!("collector{}", i));
        mint_to_address(&mut app, collector.to_string(), coins(2000000, "ibc_atom"));
        let _res = app
            .execute_contract(
                collector.clone(),
                Addr::unchecked(multi_minter_addr.clone()),
                &MinterExecuteMsg::Mint {},
                &[coin(2000000, "ibc_atom")],
            )
            .unwrap();
    }
    let collection = query_onft_collection(app.storage(), multi_minter_addr.clone());
    assert_eq!(collection.onfts.len(), 20);

    // Set time to third round
    app.set_block(BlockInfo {
        chain_id: "test_1".to_string(),
        height: 1_000,
        time: Timestamp::from_nanos(3_000_000 + 1),
    });
    // Mint nft for 5 collectors
    for i in 26..=30 {
        let collector = Addr::unchecked(format!("collector{}", i));
        mint_to_address(&mut app, collector.to_string(), coins(3000000, "ibc_atom"));
        let _res = app
            .execute_contract(
                collector.clone(),
                Addr::unchecked(multi_minter_addr.clone()),
                &MinterExecuteMsg::Mint {},
                &[coin(3000000, "ibc_atom")],
            )
            .unwrap();
    }
    let collection = query_onft_collection(app.storage(), multi_minter_addr.clone());
    assert_eq!(collection.onfts.len(), 25);
    // Set time to end of third round
    app.set_block(BlockInfo {
        chain_id: "test_1".to_string(),
        height: 1_000,
        time: Timestamp::from_nanos(4_000_000 + 1),
    });
    // Now no round is active and piublic mint is not active
    // Try minting should fail
    let res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(multi_minter_addr.clone()),
            &MinterExecuteMsg::Mint {},
            &[coin(3000000, "ibc_atom")],
        )
        .unwrap_err();
    let err = res.source().unwrap();
    let error = err
        .downcast_ref::<MultiMintOpenEditionMinterContractError>()
        .unwrap();
    assert_eq!(
        error,
        &MultiMintOpenEditionMinterContractError::WhitelistNotActive {}
    );

    // Now creator decides to add 1 more round
    let round_4 = Round {
        start_time: Timestamp::from_nanos(4_500_000),
        end_time: Timestamp::from_nanos(8_000_000),
        round_per_address_limit: 100,
        mint_price: coin(200_000, "uflix"),
    };
    let round_4_addresses = (1..=100)
        .map(|i| format!("collector{}", i))
        .collect::<Vec<String>>();
    let round_4_config = RoundConfig {
        round: round_4.clone(),
        members: round_4_addresses.clone(),
    };
    let _res = app
        .execute_contract(
            admin.clone(),
            Addr::unchecked(round_whitelist_addr.clone()),
            &RoundWhitelistExecuteMsg::AddRound {
                round_config: round_4_config,
            },
            &[],
        )
        .unwrap();

    // Set time to start of round 4
    app.set_block(BlockInfo {
        chain_id: "test_1".to_string(),
        height: 1_000,
        time: Timestamp::from_nanos(4_500_000 + 1),
    });
    // Mint nft for 25 collectors
    for i in 1..=25 {
        let collector = Addr::unchecked(format!("collector{}", i));
        mint_to_address(&mut app, collector.to_string(), coins(200000, "uflix"));
        let _res = app
            .execute_contract(
                collector.clone(),
                Addr::unchecked(multi_minter_addr.clone()),
                &MinterExecuteMsg::Mint {},
                &[coin(200000, "uflix")],
            )
            .unwrap();
    }
    let collection = query_onft_collection(app.storage(), multi_minter_addr.clone());
    assert_eq!(collection.onfts.len(), 50);

    // Now creator decides to create a new round whitelist contract with 1 round
    let rounds: Vec<Round> = vec![Round {
        start_time: Timestamp::from_nanos(8_000_000 + 2),
        end_time: Timestamp::from_nanos(11_000_000),
        round_per_address_limit: 100,
        mint_price: coin(200_000, "uflix"),
    }];

    let round_configs = rounds
        .iter()
        .enumerate()
        .map(|(_, round)| RoundConfig {
            round: round.clone(),
            members: (1..=100)
                .map(|i| Addr::unchecked(format!("collector{}", i)).to_string())
                .collect::<Vec<String>>(),
        })
        .collect::<Vec<RoundConfig>>();

    let round_whitelist_inst_msg = CreateWhitelistMsg {
        admin: admin.to_string(),
        rounds: round_configs.clone(),
    };
    let create_round_whitelist_msg =
        omniflix_round_whitelist_factory::msg::ExecuteMsg::CreateWhitelist {
            msg: round_whitelist_inst_msg,
        };
    let res = app
        .execute_contract(
            creator.clone(),
            round_whitelist_factory_addr.clone(),
            &create_round_whitelist_msg,
            &[coin(1000000, "uflix")],
        )
        .unwrap();
    let round_whitelist_addr_2 = get_contract_address_from_res(res);
    // Creator tries setting it but current one is still active
    let err = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(multi_minter_addr.clone()),
            &MinterExecuteMsg::UpdateWhitelistAddress {
                address: round_whitelist_addr_2.clone(),
            },
            &[],
        )
        .unwrap_err();
    let error = err
        .source()
        .unwrap()
        .downcast_ref::<MultiMintOpenEditionMinterContractError>()
        .unwrap();
    assert_eq!(
        error,
        &MultiMintOpenEditionMinterContractError::WhitelistAlreadyActive {}
    );
    // Set time to end of round 3
    app.set_block(BlockInfo {
        chain_id: "test_1".to_string(),
        height: 1_000,
        time: Timestamp::from_nanos(8_000_000 + 1),
    });
    // Now current whitelist is not active
    // Creator can change whitelist address
    let _res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(multi_minter_addr.clone()),
            &MinterExecuteMsg::UpdateWhitelistAddress {
                address: round_whitelist_addr_2.clone(),
            },
            &[],
        )
        .unwrap();
    // Set time to start of round 1 of new whitelist
    app.set_block(BlockInfo {
        chain_id: "test_1".to_string(),
        height: 1_000,
        time: Timestamp::from_nanos(8_000_000 + 1 + 1),
    });
    // Mint nft for 25 collectors
    for i in 1..=25 {
        let collector = Addr::unchecked(format!("collector{}", i));
        mint_to_address(&mut app, collector.to_string(), coins(200000, "uflix"));
        let _res = app
            .execute_contract(
                collector.clone(),
                Addr::unchecked(multi_minter_addr.clone()),
                &MinterExecuteMsg::Mint {},
                &[coin(200000, "uflix")],
            )
            .unwrap();
    }
    let collection = query_onft_collection(app.storage(), multi_minter_addr.clone());
    assert_eq!(collection.onfts.len(), 75);

    // Set time to start of public mint
    // New whitelist is active but public mint should override it
    app.set_block(BlockInfo {
        chain_id: "test_1".to_string(),
        height: 1_000,
        time: Timestamp::from_nanos(10_000_000 + 1),
    });
    // Mint nft for 10 collectors
    for i in 1..=10 {
        let collector = Addr::unchecked(format!("collector{}", i));
        mint_to_address(&mut app, collector.to_string(), coins(5000000, "uflix"));
        let _res = app
            .execute_contract(
                collector.clone(),
                Addr::unchecked(multi_minter_addr.clone()),
                &MinterExecuteMsg::Mint {},
                &[coin(5000000, "uflix")],
            )
            .unwrap();
    }
    let collection = query_onft_collection(app.storage(), multi_minter_addr.clone());
    assert_eq!(collection.onfts.len(), 85);
    // Creator lowers the public price to 1_000_000 uflix but makes a mistake and sets it to 1 uflix
    let _res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(multi_minter_addr.clone()),
            &MinterExecuteMsg::UpdateMintPrice {
                mint_price: coin(1, "uflix"),
            },
            &[],
        )
        .unwrap();
    // Query mint price
    let minter_config: Config = app
        .wrap()
        .query_wasm_smart(
            multi_minter_addr.clone(),
            &MultiMintOpenEditionMinterQueryMsg::Config {},
        )
        .unwrap();
    assert_eq!(minter_config.mint_price, coin(1, "uflix"));

    // Creator quickly pauses the contract
    let _res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(multi_minter_addr.clone()),
            &MinterExecuteMsg::Pause {},
            &[],
        )
        .unwrap();
    // One collector tries to mint
    let err = app
        .execute_contract(
            Addr::unchecked("collector"),
            Addr::unchecked(multi_minter_addr.clone()),
            &MinterExecuteMsg::Mint {},
            &[coin(1, "uflix")],
        )
        .unwrap_err();
    let error = err
        .source()
        .unwrap()
        .downcast_ref::<MultiMintOpenEditionMinterContractError>()
        .unwrap();
    assert_eq!(
        error,
        &MultiMintOpenEditionMinterContractError::Pause(pauser::PauseError::Paused {})
    );
    // Creator fixes the price to 1_000_000 uflix
    let _res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(multi_minter_addr.clone()),
            &MinterExecuteMsg::UpdateMintPrice {
                mint_price: coin(1_000_000, "uflix"),
            },
            &[],
        )
        .unwrap();
    // Creator decides creator 2 should also be a pauser
    // This also unpauses the contract
    let _res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(multi_minter_addr.clone()),
            &MinterExecuteMsg::SetPausers {
                pausers: vec![creator.clone().to_string(), "creator_2".to_string()],
            },
            &[],
        )
        .unwrap();
    // Creator also decides to change royalty ratio to 20%
    let _res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(multi_minter_addr.clone()),
            &MinterExecuteMsg::UpdateRoyaltyRatio {
                ratio: Decimal::percent(20).to_string(),
            },
            &[],
        )
        .unwrap();
    // Creator also changes payment collector and admin addresses
    let new_payment_collector = Addr::unchecked("new_payment_collector");
    let new_admin = Addr::unchecked("new_admin");
    let _res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(multi_minter_addr.clone()),
            &MinterExecuteMsg::UpdatePaymentCollector {
                payment_collector: new_payment_collector.clone().to_string(),
            },
            &[],
        )
        .unwrap();
    let _res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(multi_minter_addr.clone()),
            &MinterExecuteMsg::UpdateAdmin {
                admin: new_admin.clone().into_string(),
            },
            &[],
        )
        .unwrap();
    // Creator 2 decides create a new mint_instance
    let new_token_details = TokenDetails {
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
    let new_config = Config {
        mint_price: coin(5_000_000, "uflix"),
        start_time: Timestamp::from_nanos(20_000_000),
        end_time: Some(Timestamp::from_nanos(50_500_000)),
        per_address_limit: Some(100),
        whitelist_address: None,
        num_tokens: Some(100),
    };
    let new_mint_instance_msg = MultiMintOpenEditionMinterExecuteMsg::NewMintInstance {
        token_details: new_token_details,
        config: new_config,
    };
    let _res = app
        .execute_contract(
            new_admin.clone(),
            Addr::unchecked(multi_minter_addr.clone()),
            &new_mint_instance_msg,
            &[],
        )
        .unwrap();
    // New mint_instance is created

    // Try minting for older mint_instance.
    // Public mint is active
    // Public mint price was 5_000_000 uflix but creator changed it to 1_000_000 uflix
    // Creator also changed payment collector and admin addresses

    let payment_collector_before_balance = app
        .wrap()
        .query_balance(new_payment_collector.clone(), "uflix")
        .unwrap();

    let _res = app
        .execute_contract(
            collector.clone(),
            Addr::unchecked(multi_minter_addr.clone()),
            &MultiMintOpenEditionMinterExecuteMsg::Mint { mint_instance_id: Some(1) },
            &[coin(1000000, "uflix")],
        )
        .unwrap();
    let payment_collector_after_balance = app
        .wrap()
        .query_balance(new_payment_collector.clone(), "uflix")
        .unwrap();
    assert_eq!(
        payment_collector_after_balance.amount - payment_collector_before_balance.amount,
        Uint128::from(1000000u128)
    );
    let collection = query_onft_collection(app.storage(), multi_minter_addr.clone());
    assert_eq!(collection.onfts.len(), 86);
    // Now first mint_instance is mintable but second mint_instance is not mintable because public mint is not active
    let res = app
        .execute_contract(
            collector.clone(),
            Addr::unchecked(multi_minter_addr.clone()),
            &MultiMintOpenEditionMinterExecuteMsg::Mint { mint_instance_id: Some(2) },
            &[coin(1000000, "uflix")],
        )
        .unwrap_err();
    let error = res.source().unwrap();
    let error = error
        .downcast_ref::<MultiMintOpenEditionMinterContractError>()
        .unwrap();
    assert_eq!(
        error,
        &MultiMintOpenEditionMinterContractError::MintingNotStarted {
            start_time: Timestamp::from_nanos(20_000_000),
            current_time: Timestamp::from_nanos(10_000_000 + 1)
        }
    );
    // Set time to start of public mint
    app.set_block(BlockInfo {
        chain_id: "test_1".to_string(),
        height: 1_000,
        time: Timestamp::from_nanos(20_000_000 + 1),
    });
    // Now second mint_instance is mintable
    let _res = app
        .execute_contract(
            collector.clone(),
            Addr::unchecked(multi_minter_addr.clone()),
            &MultiMintOpenEditionMinterExecuteMsg::Mint { mint_instance_id: Some(2) },
            &[coin(5000000, "uflix")],
        )
        .unwrap();
    let collection = query_onft_collection(app.storage(), multi_minter_addr.clone());
    assert_eq!(collection.onfts.len(), 87);
    assert_eq!(
        collection.onfts[86].metadata.as_ref().unwrap().name,
        "MintInstance number 2 #1".to_string()
    );
}
