#![cfg(test)]
use cosmwasm_std::Decimal;
use cosmwasm_std::{coin, coins, Addr, BlockInfo, Timestamp, Uint128};
use cw_multi_test::Executor;
use minter_types::collection_details::CollectionDetails;
use minter_types::token_details::TokenDetails;
use minter_types::types::AuthDetails;
use omniflix_minter::msg::ExecuteMsg as MinterExecuteMsg;
use omniflix_minter_factory::msg::CreateMinterMsg;
use omniflix_minter_factory::msg::ExecuteMsg as FactoryExecuteMsg;
use omniflix_minter_factory::msg::MinterInitExtention;

use omniflix_round_whitelist::msg::ExecuteMsg as RoundWhitelistExecuteMsg;
use whitelist_types::{CreateWhitelistMsg, Round};
use whitelist_types::{RoundConfig, RoundWhitelistQueryMsgs};

use crate::helpers::mock_messages::factory_mock_messages::{
    return_minter_factory_inst_message, return_round_whitelist_factory_inst_message,
};
use crate::helpers::utils::{get_contract_address_from_res, mint_to_address};

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

    let res = setup();
    let admin = res.test_accounts.admin;
    let creator = res.test_accounts.creator;
    let _collector = res.test_accounts.collector;
    let minter_factory_code_id = res.minter_factory_code_id;
    let minter_code_id = res.minter_code_id;
    let round_whitelist_factory_code_id = res.round_whitelist_factory_code_id;
    let round_whitelist_code_id = res.round_whitelist_code_id;
    let mut app = res.app;

    let minter_factory_inst_msg = return_minter_factory_inst_message(minter_code_id);
    let minter_factory_addr = app
        .instantiate_contract(
            minter_factory_code_id,
            admin.clone(),
            &minter_factory_inst_msg,
            &[],
            "factory",
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
            round_per_address_limit: 1,
            mint_price: coin(1_000_000, "uflix"),
        },
        Round {
            start_time: Timestamp::from_nanos(2_000_000),
            end_time: Timestamp::from_nanos(3_000_000),
            round_per_address_limit: 1,
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
        token_details: Some(TokenDetails {
            transferable: true,
            token_name: "token_name".to_string(),
            description: Some("description".to_string()),
            base_token_uri: "base_token_uri".to_string(),
            preview_uri: Some("preview_uri".to_string()),
            extensible: true,
            nsfw: false,
            royalty_ratio: Decimal::percent(10),
            data: None,
        }),
        init: Some(MinterInitExtention {
            mint_price: coin(5_000_000, "uflix"),
            start_time: Timestamp::from_nanos(1_000_000_000),
            end_time: Some(Timestamp::from_nanos(2_000_000_000)),
            per_address_limit: Some(1),
            whitelist_address: Some(round_whitelist_addr.clone()),
            num_tokens: 100,
        }),
        auth_details: AuthDetails {
            admin: Addr::unchecked("creator".to_string()),
            payment_collector: Addr::unchecked("creator".to_string()),
        },
    };
    let mut minter_2_inst_msg = minter_1_inst_message.clone();
    let mut init = minter_2_inst_msg.init.clone().unwrap();
    init.mint_price = coin(10_000_000, "uflix");
    init.start_time = Timestamp::from_nanos(2_000_000_000);
    init.end_time = Some(Timestamp::from_nanos(3_000_000_000));
    init.per_address_limit = Some(100);
    init.num_tokens = 100;
    minter_2_inst_msg.init = Some(init);
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
                members: vec![collector_1.clone().to_string()],
                round_index: index as u8,
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
