#![cfg(test)]
use cosmwasm_std::{
    coin, to_json_binary, Addr, BlockInfo, Coin, QueryRequest, Timestamp, WasmQuery,
};

use cw_multi_test::Executor;
use cw_utils::PaymentError;
use minter_types::token_details::Token;
use minter_types::types::UserDetails;
use omniflix_open_edition_minter_factory::msg::ExecuteMsg as OpenEditionMinterFactoryExecuteMsg;
use whitelist_types::CreateWhitelistMsg;

use crate::helpers::mock_messages::whitelist_mock_messages::return_rounds;
use crate::helpers::utils::get_contract_address_from_res;

use crate::helpers::mock_messages::factory_mock_messages::{
    return_open_edition_minter_factory_inst_message, return_round_whitelist_factory_inst_message,
};

use crate::helpers::mock_messages::oem_mock_messages::return_open_edition_minter_inst_msg;

use crate::helpers::utils::query_onft_collection;

use crate::helpers::setup::setup;
use omniflix_open_edition_minter::msg::OEMQueryExtension;

use minter_types::msg::QueryMsg as BaseMinterQueryMsg;

use omniflix_open_edition_minter::msg::ExecuteMsg as OpenEditionMinterExecuteMsg;

use omniflix_open_edition_minter::error::ContractError as OpenEditionMinterError;

use omniflix_round_whitelist::error::ContractError as RoundWhitelistError;

type OpenEditionMinterQueryMsg = BaseMinterQueryMsg<OEMQueryExtension>;

#[test]
fn private_minting() {
    let res = setup();
    let admin = res.test_accounts.admin;
    let creator = res.test_accounts.creator;
    let collector = res.test_accounts.collector;
    let open_edition_minter_factory_code_id = res.open_edition_minter_factory_code_id;
    let open_edition_minter_code_id = res.open_edition_minter_code_id;
    let round_whitelist_factory_code_id = res.round_whitelist_factory_code_id;
    let round_whitelist_code_id = res.round_whitelist_code_id;
    let mut app = res.app;

    // Instantiate the oem minter factory
    let open_edition_minter_factory_instantiate_msg =
        return_open_edition_minter_factory_inst_message(open_edition_minter_code_id, None);

    let open_edition_minter_factory_address = app
        .instantiate_contract(
            open_edition_minter_factory_code_id,
            admin.clone(),
            &open_edition_minter_factory_instantiate_msg.clone(),
            &[],
            "Open Edition Minter Factory",
            None,
        )
        .unwrap();

    // Create a whitelist factory
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

    let rounds = return_rounds();

    let round_whitelist_inst_msg = CreateWhitelistMsg {
        admin: admin.to_string(),
        rounds: rounds.clone(),
    };
    let create_round_whitelist_msg =
        omniflix_round_whitelist_factory::msg::ExecuteMsg::CreateWhitelist {
            msg: round_whitelist_inst_msg,
        };
    // Create a whitelist
    let res = app
        .execute_contract(
            admin.clone(),
            round_whitelist_factory_addr,
            &create_round_whitelist_msg,
            &[coin(1000000, "uflix")],
        )
        .unwrap();
    let whitelist_address = get_contract_address_from_res(res);

    // Create a minter
    let mut open_edition_minter_instantiate_msg = return_open_edition_minter_inst_msg();
    open_edition_minter_instantiate_msg.init.whitelist_address = Some(whitelist_address.clone());

    // Set block time to rounds start time and create a oem
    // Should fail because the whitelist is active
    app.set_block(BlockInfo {
        chain_id: "test_1".to_string(),
        height: 1_000,
        time: Timestamp::from_nanos(2000 + 1),
    });

    let create_minter_msg = OpenEditionMinterFactoryExecuteMsg::CreateOpenEditionMinter {
        msg: open_edition_minter_instantiate_msg,
    };

    let res = app
        .execute_contract(
            creator.clone(),
            open_edition_minter_factory_address.clone(),
            &create_minter_msg,
            &[Coin::new(2000000, "uflix")],
        )
        .unwrap_err();
    let error = res.source().unwrap().source().unwrap();
    let error = error.downcast_ref::<OpenEditionMinterError>().unwrap();
    assert_eq!(error, &OpenEditionMinterError::WhitelistAlreadyActive {});

    // Reset block time to default
    app.set_block(BlockInfo {
        chain_id: "test_1".to_string(),
        height: 1_000,
        time: Timestamp::from_nanos(1_000),
    });

    // Create a minter
    let mut open_edition_minter_instantiate_msg = return_open_edition_minter_inst_msg();
    open_edition_minter_instantiate_msg.init.whitelist_address = Some(whitelist_address.clone());
    let create_minter_msg = OpenEditionMinterFactoryExecuteMsg::CreateOpenEditionMinter {
        msg: open_edition_minter_instantiate_msg.clone(),
    };

    let res = app
        .execute_contract(
            creator.clone(),
            open_edition_minter_factory_address.clone(),
            &create_minter_msg,
            &[Coin::new(2000000, "uflix")],
        )
        .unwrap();
    let minter_address = get_contract_address_from_res(res);

    // Try minting should fail because the whitelist no rounds are active
    let mint_msg = OpenEditionMinterExecuteMsg::Mint {};
    let res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(minter_address.clone()),
            &mint_msg,
            &[Coin::new(1000000, "uflix")],
        )
        .unwrap_err();
    let error = res.source().unwrap();
    let error = error.downcast_ref::<OpenEditionMinterError>().unwrap();
    assert_eq!(error, &OpenEditionMinterError::WhitelistNotActive {});

    let round_1_start_time = rounds[0].start_time;
    app.set_block(BlockInfo {
        chain_id: "test_1".to_string(),
        height: 1_000,
        time: round_1_start_time,
    });
    // Mint for creator should fail because the creator is not whitelisted for first round
    // Creator is also an admin for this minter but this does not matter since executed msg is not MintAdmin{}
    let mint_msg = OpenEditionMinterExecuteMsg::Mint {};
    let res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(minter_address.clone()),
            &mint_msg,
            &[Coin::new(1000000, "diffirent_denom")],
        )
        .unwrap_err();
    let error = res.source().unwrap();
    let error = error.downcast_ref::<OpenEditionMinterError>().unwrap();
    assert_eq!(error, &OpenEditionMinterError::AddressNotWhitelisted {});

    let round_1_mint_price = &rounds[0].mint_price;

    // Mint for collector
    let mint_msg = OpenEditionMinterExecuteMsg::Mint {};
    let _res = app
        .execute_contract(
            collector.clone(),
            Addr::unchecked(minter_address.clone()),
            &mint_msg.clone(),
            &[round_1_mint_price.clone()],
        )
        .unwrap();
    // Query the collection
    let collection = query_onft_collection(app.storage(), minter_address.clone());
    assert_eq!(collection.onfts[0].id, 1.to_string());
    // Query user minting details
    let user_minting_details: UserDetails = app
        .wrap()
        .query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: minter_address.clone(),
            msg: to_json_binary::<OpenEditionMinterQueryMsg>(
                &OpenEditionMinterQueryMsg::UserMintingDetails {
                    address: collector.clone().into_string(),
                },
            )
            .unwrap(),
        }))
        .unwrap();

    assert_eq!(
        user_minting_details.minted_tokens,
        [Token {
            token_id: "1".to_string(),
            migration_nft_data: None,
        }]
    );
    assert_eq!(user_minting_details.total_minted_count, 1);
    assert_eq!(user_minting_details.public_mint_count, 0);

    // Try minting for collector again should fail because the collector has reached the mint limit
    let res = app
        .execute_contract(
            collector.clone(),
            Addr::unchecked(minter_address.clone()),
            &mint_msg.clone(),
            &[round_1_mint_price.clone()],
        )
        .unwrap_err();
    let error = res.source().unwrap().source().unwrap();
    let error = error.downcast_ref::<RoundWhitelistError>().unwrap();
    assert_eq!(error, &RoundWhitelistError::RoundReachedMintLimit {});

    // Set block time to round 2 start time
    let round_2_start_time = rounds[1].start_time;
    let round_2_mint_price = &rounds[1].mint_price;

    app.set_block(BlockInfo {
        chain_id: "test_1".to_string(),
        height: 1_000,
        time: round_2_start_time,
    });

    // Mint for collector
    // Should fail because the collector is not whitelisted for round 2
    let res = app
        .execute_contract(
            collector.clone(),
            Addr::unchecked(minter_address.clone()),
            &mint_msg.clone(),
            &[round_2_mint_price.clone()],
        )
        .unwrap_err();
    let error = res.source().unwrap();
    let error = error.downcast_ref::<OpenEditionMinterError>().unwrap();
    assert_eq!(error, &OpenEditionMinterError::AddressNotWhitelisted {});

    // Mint for creator
    // Send round 1's mint price
    // Should fail because wrong mint price
    let mint_msg = OpenEditionMinterExecuteMsg::Mint {};
    let res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(minter_address.clone()),
            &mint_msg.clone(),
            &[round_1_mint_price.clone()],
        )
        .unwrap_err();
    let error = res.source().unwrap();
    let error = error.downcast_ref::<OpenEditionMinterError>().unwrap();
    assert_eq!(
        error,
        &OpenEditionMinterError::PaymentError(PaymentError::ExtraDenom(
            round_1_mint_price.clone().denom
        ))
    );

    // Mint for creator
    // Send round 2's mint price
    // Should not fail because the creator is whitelisted for round 2
    // Price is correct
    // Round limit is not reached
    let mint_msg = OpenEditionMinterExecuteMsg::Mint {};
    let _res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(minter_address.clone()),
            &mint_msg.clone(),
            &[round_2_mint_price.clone()],
        )
        .unwrap();

    // Now this whitelist has been used to its limit
    // Wait for public minting time
    let public_minting_time = &open_edition_minter_instantiate_msg.clone().init.start_time;
    app.set_block(BlockInfo {
        chain_id: "test_1".to_string(),
        height: 1_000,
        time: *public_minting_time,
    });
    let public_mint_price = &open_edition_minter_instantiate_msg.clone().init.mint_price;

    // Mint for collector
    let mint_msg = OpenEditionMinterExecuteMsg::Mint {};
    let _res = app
        .execute_contract(
            collector.clone(),
            Addr::unchecked(minter_address.clone()),
            &mint_msg,
            &[public_mint_price.clone()],
        )
        .unwrap();

    // Query the collection
    let collection = query_onft_collection(app.storage(), minter_address.clone());
    assert_eq!(collection.onfts.len(), 3);
}
