#![cfg(test)]
use cosmwasm_std::Decimal;
use cosmwasm_std::{coin, coins, Addr, BlockInfo, Timestamp, Uint128};
use cw_multi_test::Executor;
use minter_types::QueryMsg as CommonMinterQueryMsg;
use minter_types::TokenDetails;
use minter_types::{CollectionDetails, Config};
use omniflix_minter::msg::ExecuteMsg as MinterExecuteMsg;
use omniflix_minter_factory::msg::CreateMinterMsg;
use omniflix_minter_factory::msg::ExecuteMsg as FactoryExecuteMsg;
use omniflix_minter_factory::msg::MinterInitExtention;
use omniflix_multi_mint_open_edition_minter::error::ContractError as MultiMintOpenEditionMinterContractError;
use omniflix_multi_mint_open_edition_minter::msg::ExecuteMsg as MultiMintOpenEditionMinterExecuteMsg;
use omniflix_multi_mint_open_edition_minter::msg::QueryMsgExtension as MultiMintOpenEditionMinterQueryMsgExtension;

use omniflix_open_edition_minter_factory::msg::ExecuteMsg as OpenEditionMinterFactoryExecuteMsg;
use omniflix_open_edition_minter_factory::msg::{
    OpenEditionMinterCreateMsg, OpenEditionMinterInitExtention,
};
use omniflix_round_whitelist::msg::ExecuteMsg as RoundWhitelistExecuteMsg;
use whitelist_types::Round;
use whitelist_types::RoundWhitelistQueryMsgs;

type MultiMintOpenEditionMinterQueryMsg =
    CommonMinterQueryMsg<MultiMintOpenEditionMinterQueryMsgExtension>;

use crate::helpers::utils::{
    get_contract_address_from_res, mint_to_address, return_factory_inst_message,
};

use crate::{helpers::setup::setup, helpers::utils::query_onft_collection};
use omniflix_minter::error::ContractError as MinterContractError;

use omniflix_round_whitelist::error::ContractError as RoundWhitelistContractError;

#[test]
fn test_scenario_1() {
    // Scenario 1:
    // 1. Creator creates a new round whitelist contract with 3 rounds
    // Rounds: 1, 2, 3
    // Round_1:
    // Round{
    //     start_time: 1_000_000,
    //     end_time: 2_000_000,
    //     round_per_address_limit: 1,
    //     addresses: ["collector"+ 1...25],
    //     round_price : 1_000_000 uflix
    // }
    // Round_2:
    // Round{
    //     start_time: 2_000_000,
    //     end_time: 3_000_000,
    //     round_per_address_limit: 1,
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

    // 2. Creator creates 2 minter contracts sending same whitelist address
    // Minter_1:
    // {
    // public_mint_start_time: 1_000_000_000,
    // public_mint_end_time: 2_000_000_000,
    // public_mint_limit:1,
    // public_mint_price: 5_000_000 uflix,
    // supply: 100,
    //   }
    // Minter_2:
    // {
    // public_mint_start_time: 2_000_000_000,
    // public_mint_end_time: 3_000_000_000,
    // public_mint_limit: 100,
    // public_mint_price: 10_000_000 uflix,
    // supply: 100,
    //}
    // 3. Collector_1 buys 1 NFT from Minter_1 during round 1
    // 4. Collector_1 buys 1 NFT from Minter_2 during round 1
    // Expected scenario is that Collector_1 should be able to buy 1 NFT from Minter_1 and 1 NFT from Minter_2
    // Round whitelist saves collectors minted NFTs from diffirent minter contracts seperately and they dont affect each other
    // This event proves that because round 1 mint limit is 1 and collector_1 already bought 1 NFT from minter_1,
    // he cant buy another NFT from minter_1 during first round but he can buy 1 NFT from minter_2
    // 5. Collector_1 waits for round 2 to buy 1 NFT from Minter_1
    // In total Collector_1 has 2 NFTs from minter_1 and 1 NFT from minter_2
    // This proves that public mint limit is not affecting private mints because public mint limit is 1 but collector_1 has 2 NFTs from minter_1
    // 6. By the time round 3 starts, collector_1 realizes that he is not in the whitelist for round 3
    // Creator add collector_1 to round 3 whitelist addresses
    // 7. Collector_1 buys 1 NFT from Minter_1 during round 3
    // 8. When round 3 ends, minter_1 had only minted 3 NFTs so creator decides to add 1 more round
    // Round_4:
    // Round{
    //     start_time: 4_000_000,
    //     end_time: 5_000_000,
    //     round_per_address_limit: 1,
    //     addresses: ["collector"+ 1 ... 100],
    //     round_price : 200_000 uflix
    // }
    // 9. Collector_1 buys 1 NFT from Minter_1 during round 4
    // 10. Collector_1 has 4 NFTs from minter_1 and 1 NFT from minter_2
    // 11. Creator waits for public mint to start and buys 1 NFT from Minter_1
    // 12. Creator can not buy another NFT from Minter_1 because public mint limit is 1

    let (
        mut app,
        test_addresses,
        minter_factory_code_id,
        minter_code_id,
        round_whitelist_factory_code_id,
        round_whitelist_code_id,
        _open_edition_minter_code_id,
        _open_edition_minter_factory_code_id,
        _multi_mint_open_edition_minter_code_id,
    ) = setup();
    let admin = test_addresses.admin;
    let creator = test_addresses.creator;
    let _collector = test_addresses.collector;

    let factory_inst_msg = return_factory_inst_message(minter_code_id);
    let minter_factory_addr = app
        .instantiate_contract(
            minter_factory_code_id,
            admin.clone(),
            &factory_inst_msg,
            &[],
            "factory",
            None,
        )
        .unwrap();
    let round_whitelist_factory_inst_msg = return_factory_inst_message(round_whitelist_code_id);
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
            round_per_address_limit: 1,
            addresses: (1..=25)
                .map(|i| Addr::unchecked(format!("collector{}", i)))
                .collect::<Vec<Addr>>(),
            mint_price: coin(1_000_000, "uflix"),
        },
        Round {
            start_time: Timestamp::from_nanos(2_000_000),
            end_time: Timestamp::from_nanos(3_000_000),
            round_per_address_limit: 1,
            addresses: (1..=25)
                .map(|i| Addr::unchecked(format!("collector{}", i)))
                .collect::<Vec<Addr>>(),
            mint_price: coin(2_000_000, "ibc_atom"),
        },
        Round {
            start_time: Timestamp::from_nanos(3_000_000),
            end_time: Timestamp::from_nanos(4_000_000),
            round_per_address_limit: 1,
            addresses: (26..=50)
                .map(|i| Addr::unchecked(format!("collector{}", i)))
                .collect::<Vec<Addr>>(),
            mint_price: coin(3_000_000, "ibc_atom"),
        },
    ]
    .to_vec();

    let round_whitelist_inst_msg = whitelist_types::InstantiateMsg {
        admin: admin.to_string(),
        rounds: rounds.clone(),
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

    let minter_1_inst_message = CreateMinterMsg {
        collection_details: CollectionDetails {
            collection_name: "Test_collection_1".to_string(),
            description: Some("description".to_string()),
            preview_uri: Some("preview_uri".to_string()),
            schema: Some("schema".to_string()),
            symbol: "symbol".to_string(),
            id: "test1".to_string(),
            uri: Some("uri".to_string()),
            uri_hash: Some("uri_hash".to_string()),
            data: Some("data".to_string()),
            royalty_receivers: None,
        },
        token_details: TokenDetails {
            transferable: true,
            token_name: "token_name".to_string(),
            description: Some("description".to_string()),
            base_token_uri: "base_token_uri".to_string(),
            preview_uri: Some("preview_uri".to_string()),
            extensible: true,
            nsfw: false,
            royalty_ratio: Decimal::percent(10),
            data: None,
        },
        init: MinterInitExtention {
            admin: creator.to_string(),
            mint_price: coin(5_000_000, "uflix"),
            start_time: Timestamp::from_nanos(1_000_000_000),
            end_time: Some(Timestamp::from_nanos(2_000_000_000)),
            per_address_limit: Some(1),
            payment_collector: Some(creator.to_string()),
            whitelist_address: Some(round_whitelist_addr.clone()),
            num_tokens: 100,
        },
    };
    let mut minter_2_inst_msg = minter_1_inst_message.clone();
    minter_2_inst_msg.init.mint_price = coin(10_000_000, "uflix");
    minter_2_inst_msg.init.start_time = Timestamp::from_nanos(2_000_000_000);
    minter_2_inst_msg.init.end_time = Some(Timestamp::from_nanos(3_000_000_000));
    minter_2_inst_msg.init.per_address_limit = Some(100);
    minter_2_inst_msg.init.num_tokens = 100;
    minter_2_inst_msg.collection_details.id = "test2".to_string();
    minter_2_inst_msg.collection_details.collection_name = "Test_collection_2".to_string();

    // Instantiate minter_1
    let res = app
        .execute_contract(
            creator.clone(),
            minter_factory_addr.clone(),
            &FactoryExecuteMsg::CreateMinter {
                msg: minter_1_inst_message.clone(),
            },
            &[coin(2000000, "uflix")],
        )
        .unwrap();
    let minter_1_addr = get_contract_address_from_res(res);

    // Instantiate minter_2
    let res = app
        .execute_contract(
            creator.clone(),
            minter_factory_addr.clone(),
            &FactoryExecuteMsg::CreateMinter {
                msg: minter_2_inst_msg.clone(),
            },
            &[coin(2000000, "uflix")],
        )
        .unwrap();
    let minter_2_addr = get_contract_address_from_res(res);

    // Collector_1 buys 1 NFT from Minter_1 during round 1
    // Price is 1_000_000 uflix
    let collector_1 = Addr::unchecked("collector1");
    // Mint flix to collector_1
    mint_to_address(
        &mut app,
        collector_1.to_string(),
        coins(1000000 + 1000000, "uflix"),
    );
    app.set_block(BlockInfo {
        chain_id: "test_1".to_string(),
        height: 1_000,
        time: Timestamp::from_nanos(1_000_000 + 1),
    });
    let _res = app
        .execute_contract(
            collector_1.clone(),
            Addr::unchecked(minter_1_addr.clone()),
            &MinterExecuteMsg::Mint {},
            &[coin(1000000, "uflix")],
        )
        .unwrap();

    // Collector_1 buys 1 NFT from Minter_2 during round 1
    // Price is round 1 price and its 1_000_000 uflix
    let _res = app
        .execute_contract(
            collector_1.clone(),
            Addr::unchecked(minter_2_addr.clone()),
            &MinterExecuteMsg::Mint {},
            &[coin(1000000, "uflix")],
        )
        .unwrap();

    // Collector 1 depleted his funds
    let balance = app
        .wrap()
        .query_balance(collector_1.clone(), "uflix")
        .unwrap();
    assert_eq!(balance.amount, Uint128::zero());

    // Query Onft
    let collection_1 = query_onft_collection(app.storage(), minter_1_addr.clone());
    assert_eq!(collection_1.onfts.len(), 1);
    let collection_2 = query_onft_collection(app.storage(), minter_2_addr.clone());
    assert_eq!(collection_2.onfts.len(), 1);

    // Collector one tries to buy another NFT from minter_1 during round 1
    // This should fail because he already bought 1 NFT from minter_1 during round 1
    // Mint flix to collector_1
    mint_to_address(&mut app, collector_1.to_string(), coins(1000000, "uflix"));
    let res = app
        .execute_contract(
            collector_1.clone(),
            Addr::unchecked(minter_1_addr.clone()),
            &MinterExecuteMsg::Mint {},
            &[coin(1000000, "uflix")],
        )
        .unwrap_err();
    let err = res.source().unwrap().source().unwrap();
    let error = err.downcast_ref::<RoundWhitelistContractError>().unwrap();
    assert_eq!(
        error,
        &RoundWhitelistContractError::RoundReachedMintLimit {}
    );

    // Collector_1 waits for round 2 to buy 1 NFT from Minter_1
    // Price is 2_000_000 ibc_atom
    app.set_block(BlockInfo {
        chain_id: "test_1".to_string(),
        height: 1_000,
        time: Timestamp::from_nanos(2_000_000 + 1),
    });
    // Mint ibc_atom to collector_1
    mint_to_address(
        &mut app,
        collector_1.to_string(),
        coins(2000000, "ibc_atom"),
    );
    let _res = app
        .execute_contract(
            collector_1.clone(),
            Addr::unchecked(minter_1_addr.clone()),
            &MinterExecuteMsg::Mint {},
            &[coin(2000000, "ibc_atom")],
        )
        .unwrap();

    // Collector_1 depleted his funds
    let balance = app
        .wrap()
        .query_balance(collector_1.clone(), "ibc_atom")
        .unwrap();
    assert_eq!(balance.amount, Uint128::zero());

    // Query Onft
    let collection_1 = query_onft_collection(app.storage(), minter_1_addr.clone());
    assert_eq!(collection_1.onfts.len(), 2);

    // Wait for round 3 to start
    app.set_block(BlockInfo {
        chain_id: "test_1".to_string(),
        height: 1_000,
        time: Timestamp::from_nanos(3_000_000 + 1),
    });
    // Mint ibc_atom to collector_1
    mint_to_address(
        &mut app,
        collector_1.to_string(),
        coins(3000000, "ibc_atom"),
    );
    // Collector_1 buys 1 NFT from Minter_1 during round 3
    // Price is 3_000_000 ibc_atom
    // Should fail because collector_1 is not in the whitelist for round 3
    let res = app
        .execute_contract(
            collector_1.clone(),
            Addr::unchecked(minter_1_addr.clone()),
            &MinterExecuteMsg::Mint {},
            &[coin(3000000, "ibc_atom")],
        )
        .unwrap_err();
    let err = res.source().unwrap();
    let error = err.downcast_ref::<MinterContractError>().unwrap();
    assert_eq!(error, &MinterContractError::AddressNotWhitelisted {});

    // Add collector_1 to round 3 whitelist addresses
    let rounds: Vec<(u32, Round)> = app
        .wrap()
        .query_wasm_smart(
            round_whitelist_addr.clone(),
            &RoundWhitelistQueryMsgs::Rounds {},
        )
        .unwrap();

    // Found the index of round 3(Spoiler alert: its 3)
    let index = rounds
        .iter()
        .find(|(_index, round)| round.start_time == Timestamp::from_nanos(3_000_000))
        .unwrap()
        .0;

    // Add collector_1 to round 3 whitelist addresses
    let _res = app
        .execute_contract(
            admin.clone(),
            Addr::unchecked(round_whitelist_addr.clone()),
            &RoundWhitelistExecuteMsg::AddMembers {
                address: vec![collector_1.clone().to_string()],
                round_index: index as u32,
            },
            &[],
        )
        .unwrap();

    // Collector_1 buys 1 NFT from Minter_1 during round 3
    let _res = app
        .execute_contract(
            collector_1.clone(),
            Addr::unchecked(minter_1_addr.clone()),
            &MinterExecuteMsg::Mint {},
            &[coin(3000000, "ibc_atom")],
        )
        .unwrap();

    // Query Onft
    let collection_1 = query_onft_collection(app.storage(), minter_1_addr.clone());
    assert_eq!(collection_1.onfts.len(), 3);

    // Creator adds 1 more round
    let round_4 = Round {
        start_time: Timestamp::from_nanos(4_000_000),
        end_time: Timestamp::from_nanos(5_000_000),
        round_per_address_limit: 1,
        addresses: (1..=100)
            .map(|i| Addr::unchecked(format!("collector{}", i)))
            .collect::<Vec<Addr>>(),
        mint_price: coin(200_000, "uflix"),
    };
    let _res = app
        .execute_contract(
            admin.clone(),
            Addr::unchecked(round_whitelist_addr.clone()),
            &RoundWhitelistExecuteMsg::AddRound { round: round_4 },
            &[],
        )
        .unwrap();
    // Query rounds
    let rounds: Vec<(u32, Round)> = app
        .wrap()
        .query_wasm_smart(
            round_whitelist_addr.clone(),
            &RoundWhitelistQueryMsgs::Rounds {},
        )
        .unwrap();
    assert_eq!(rounds.len(), 4);

    // Wait for round 4 to start
    app.set_block(BlockInfo {
        chain_id: "test_1".to_string(),
        height: 1_000,
        time: Timestamp::from_nanos(4_000_000 + 1),
    });
    // Collector_1 buys 1 NFT from Minter_1 during round 4
    // Price is 200_000 uflix
    // Mint flix to collector_1
    mint_to_address(&mut app, collector_1.to_string(), coins(200000, "uflix"));
    let _res = app
        .execute_contract(
            collector_1.clone(),
            Addr::unchecked(minter_1_addr.clone()),
            &MinterExecuteMsg::Mint {},
            &[coin(200000, "uflix")],
        )
        .unwrap();

    // Query Onft
    let collection_1 = query_onft_collection(app.storage(), minter_1_addr.clone());
    assert_eq!(collection_1.onfts.len(), 4);

    // Collector_1 waits for public mint to start and buys 1 NFT from Minter_1
    // Price is 5_000_000 uflix
    app.set_block(BlockInfo {
        chain_id: "test_1".to_string(),
        height: 1_000,
        time: Timestamp::from_nanos(1_000_000_000 + 1),
    });
    // Mint flix to
    mint_to_address(&mut app, collector_1.to_string(), coins(5000000, "uflix"));

    // Creator buys 1 NFT from Minter_1
    let _res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(minter_1_addr.clone()),
            &MinterExecuteMsg::Mint {},
            &[coin(5000000, "uflix")],
        )
        .unwrap();
}

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
    // Now Creator 2 decides create a new drop
    let (
        mut app,
        test_addresses,
        _minter_factory_code_id,
        _minter_code_id,
        round_whitelist_factory_code_id,
        round_whitelist_code_id,
        open_edition_minter_factory_code_id,
        _open_edition_minter_code_id,
        multi_minter_code_id,
    ) = setup();
    let admin = test_addresses.admin;
    let creator = test_addresses.creator;
    let collector = test_addresses.collector;
    // Instantiate the minter factory
    let open_edition_minter_factory_instantiate_msg =
        return_factory_inst_message(multi_minter_code_id);

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
    let round_whitelist_factory_inst_msg = return_factory_inst_message(round_whitelist_code_id);
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
            addresses: (1..=25)
                .map(|i| Addr::unchecked(format!("collector{}", i)))
                .collect::<Vec<Addr>>(),
            mint_price: coin(1_000_000, "uflix"),
        },
        Round {
            start_time: Timestamp::from_nanos(2_000_000),
            end_time: Timestamp::from_nanos(3_000_000),
            round_per_address_limit: 100,
            addresses: (1..=25)
                .map(|i| Addr::unchecked(format!("collector{}", i)))
                .collect::<Vec<Addr>>(),
            mint_price: coin(2_000_000, "ibc_atom"),
        },
        Round {
            start_time: Timestamp::from_nanos(3_000_000),
            end_time: Timestamp::from_nanos(4_000_000),
            round_per_address_limit: 1,
            addresses: (26..=50)
                .map(|i| Addr::unchecked(format!("collector{}", i)))
                .collect::<Vec<Addr>>(),
            mint_price: coin(3_000_000, "ibc_atom"),
        },
    ]
    .to_vec();

    let round_whitelist_inst_msg = whitelist_types::InstantiateMsg {
        admin: admin.to_string(),
        rounds: rounds.clone(),
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
        schema: Some("Some schema for drop 1".to_string()),
        symbol: "MMOEM".to_string(),
        id: "MMOEM test 1".to_string(),
        uri: Some("Some uri".to_string()),
        uri_hash: Some("uri_hash".to_string()),
        data: Some("data".to_string()),
        royalty_receivers: None,
    };
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
    let init = OpenEditionMinterInitExtention {
        admin: creator.to_string(),
        mint_price: coin(5_000_000, "uflix"),
        start_time: Timestamp::from_nanos(10_000_000),
        end_time: Some(Timestamp::from_nanos(50_500_000)),
        per_address_limit: Some(100),
        payment_collector: None,
        whitelist_address: Some(round_whitelist_addr.clone()),
        num_tokens: Some(100),
    };

    let multi_minter_inst_msg = OpenEditionMinterCreateMsg {
        collection_details,
        init,
        token_details,
    };

    let res = app
        .execute_contract(
            creator.clone(),
            open_edition_minter_factory_address.clone(),
            &OpenEditionMinterFactoryExecuteMsg::CreateMinter {
                msg: multi_minter_inst_msg,
            },
            &[coin(2000000, "uflix")],
        )
        .unwrap();
    let multi_minter_addr = get_contract_address_from_res(res);

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
        addresses: (1..=100)
            .map(|i| Addr::unchecked(format!("collector{}", i)))
            .collect::<Vec<Addr>>(),
        mint_price: coin(200_000, "uflix"),
    };
    let _res = app
        .execute_contract(
            admin.clone(),
            Addr::unchecked(round_whitelist_addr.clone()),
            &RoundWhitelistExecuteMsg::AddRound { round: round_4 },
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
        addresses: (1..=100)
            .map(|i| Addr::unchecked(format!("collector{}", i)))
            .collect::<Vec<Addr>>(),
        mint_price: coin(200_000, "uflix"),
    }];
    let round_whitelist_inst_msg = whitelist_types::InstantiateMsg {
        admin: admin.to_string(),
        rounds: rounds.clone(),
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
            &MinterExecuteMsg::SetPaymentCollector {
                payment_collector: new_payment_collector.clone().to_string(),
            },
            &[],
        )
        .unwrap();
    let _res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(multi_minter_addr.clone()),
            &MinterExecuteMsg::SetAdmin {
                admin: new_admin.clone().into_string(),
            },
            &[],
        )
        .unwrap();
    // Creator 2 decides create a new drop
    let new_token_details = TokenDetails {
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
    let new_config = Config {
        mint_price: coin(5_000_000, "uflix"),
        start_time: Timestamp::from_nanos(20_000_000),
        end_time: Some(Timestamp::from_nanos(50_500_000)),
        per_address_limit: Some(100),
        whitelist_address: None,
        num_tokens: Some(100),
    };
    let new_drop_msg = MultiMintOpenEditionMinterExecuteMsg::NewDrop {
        new_token_details,
        new_config,
    };
    let _res = app
        .execute_contract(
            new_admin.clone(),
            Addr::unchecked(multi_minter_addr.clone()),
            &new_drop_msg,
            &[],
        )
        .unwrap();
    // New drop is created

    // Try minting for older drop.
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
            &MultiMintOpenEditionMinterExecuteMsg::Mint { drop_id: Some(1) },
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
    // Now first drop is mintable but second drop is not mintable because public mint is not active
    let res = app
        .execute_contract(
            collector.clone(),
            Addr::unchecked(multi_minter_addr.clone()),
            &MultiMintOpenEditionMinterExecuteMsg::Mint { drop_id: Some(2) },
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
    // Now second drop is mintable
    let _res = app
        .execute_contract(
            collector.clone(),
            Addr::unchecked(multi_minter_addr.clone()),
            &MultiMintOpenEditionMinterExecuteMsg::Mint { drop_id: Some(2) },
            &[coin(5000000, "uflix")],
        )
        .unwrap();
    let collection = query_onft_collection(app.storage(), multi_minter_addr.clone());
    assert_eq!(collection.onfts.len(), 87);
    assert_eq!(
        collection.onfts[86].metadata.as_ref().unwrap().name,
        "Drop number 2 # 1".to_string()
    );
}
