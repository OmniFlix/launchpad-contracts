use cosmwasm_std::Addr;
use cosmwasm_std::{coin, BlockInfo, Timestamp, Uint128};
use cw_multi_test::Executor;

use minter_types::types::UserDetails;

use minter_types::msg::QueryMsg;

use omniflix_minter_factory::msg::ExecuteMsg as FactoryExecuteMsg;
use whitelist_types::CreateWhitelistMsg;

use crate::helpers::mock_messages::factory_mock_messages::{
    return_minter_factory_inst_message, return_round_whitelist_factory_inst_message,
};
use crate::helpers::mock_messages::minter_mock_messages::return_minter_instantiate_msg;
use crate::helpers::mock_messages::whitelist_mock_messages::return_rounds;
use crate::helpers::utils::get_contract_address_from_res;

use crate::{helpers::setup::setup, helpers::utils::query_onft_collection};
use omniflix_minter::error::ContractError as MinterContractError;
use omniflix_minter::msg::ExecuteMsg as MinterExecuteMsg;
use omniflix_minter::msg::MinterExtensionQueryMsg;

use omniflix_round_whitelist::error::ContractError as RoundWhitelistContractError;
type MinterQueryMsg = QueryMsg<MinterExtensionQueryMsg>;

#[test]
fn minter_private_minting() {
    let res = setup();
    let admin = res.test_accounts.admin;
    let creator = res.test_accounts.creator;
    let collector = res.test_accounts.collector;
    let minter_factory_code_id = res.minter_factory_code_id;
    let minter_code_id = res.minter_code_id;
    let round_whitelist_factory_code_id = res.round_whitelist_factory_code_id;
    let round_whitelist_code_id = res.round_whitelist_code_id;
    let mut app = res.app;

    let factory_inst_msg = return_minter_factory_inst_message(minter_code_id);
    let factory_addr = app
        .instantiate_contract(
            minter_factory_code_id,
            admin.clone(),
            &factory_inst_msg,
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

    let rounds = return_rounds();
    let round_1_price = rounds[0].mint_price.clone();
    let round_2_price = rounds[1].mint_price.clone();

    // Now is 1_000 as default
    let round_whitelist_inst_msg = CreateWhitelistMsg {
        admin: admin.to_string(),
        rounds: rounds.clone(),
    };
    let create_round_whitelist_msg =
        omniflix_round_whitelist_factory::msg::ExecuteMsg::CreateWhitelist {
            msg: round_whitelist_inst_msg,
        };
    let res = app
        .execute_contract(
            admin.clone(),
            round_whitelist_factory_addr,
            &create_round_whitelist_msg,
            &[coin(1000000, "uflix")],
        )
        .unwrap();

    let round_whitelist_address = get_contract_address_from_res(res.clone());

    let mut minter_inst_msg = return_minter_instantiate_msg();
    let mut init = minter_inst_msg.init.unwrap();
    init.whitelist_address = Some(round_whitelist_address.clone());
    init.per_address_limit = Some(1);
    minter_inst_msg.init = Some(init);

    let create_minter_msg = FactoryExecuteMsg::CreateMinter {
        msg: minter_inst_msg,
    };

    let res = app
        .execute_contract(
            admin.clone(),
            factory_addr,
            &create_minter_msg.clone(),
            &[coin(2000000, "uflix")],
        )
        .unwrap();
    let minter_address = get_contract_address_from_res(res.clone());

    // Try minting when whitelist is not active
    let error = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(minter_address.clone()),
            &MinterExecuteMsg::Mint {},
            &[coin(1000000, "uflix")],
        )
        .unwrap_err();
    let res = error.source().unwrap();
    let error = res.downcast_ref::<MinterContractError>().unwrap();
    assert_eq!(error, &MinterContractError::WhitelistNotActive {});

    // Try minting with non whitelisted address
    app.set_block(BlockInfo {
        chain_id: "test_1".to_string(),
        height: 1_000,
        time: Timestamp::from_nanos(2_000 + 1),
    });

    let error = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(minter_address.clone()),
            &MinterExecuteMsg::Mint {},
            &[coin(1000000, "uflix")],
        )
        .unwrap_err();
    let res = error.source().unwrap();
    let error = res.downcast_ref::<MinterContractError>().unwrap();
    assert_eq!(error, &MinterContractError::AddressNotWhitelisted {});

    // Try minting round one with wrong denom
    let error = app
        .execute_contract(
            collector.clone(),
            Addr::unchecked(minter_address.clone()),
            &MinterExecuteMsg::Mint {},
            &[coin(1000000, "uflix")],
        )
        .unwrap_err();
    let res = error.source().unwrap();
    let error = res.downcast_ref::<MinterContractError>().unwrap();
    assert_eq!(
        error,
        &MinterContractError::PaymentError(cw_utils::PaymentError::ExtraDenom("uflix".to_string()))
    );
    // Try minting round one with wrong amount
    let error = app
        .execute_contract(
            collector.clone(),
            Addr::unchecked(minter_address.clone()),
            &MinterExecuteMsg::Mint {},
            &[coin(100000 + 1, "diffirent_denom")],
        )
        .unwrap_err();
    let res = error.source().unwrap();
    let error = res.downcast_ref::<MinterContractError>().unwrap();
    assert_eq!(
        error,
        &MinterContractError::IncorrectPaymentAmount {
            expected: Uint128::from(1000000u128),
            sent: Uint128::from(100001u128)
        }
    );

    // Try minting round one with correct denom and amount
    let res = app
        .execute_contract(
            collector.clone(),
            Addr::unchecked(minter_address.clone()),
            &MinterExecuteMsg::Mint {},
            &[round_1_price.clone()],
        )
        .unwrap();
    let token_id: String = res.events[1].attributes[2].value.clone();
    let collection_id: String = res.events[1].attributes[3].value.clone();
    // We are quering collection to check if it is minted from our mocked onft keeper
    let collection = query_onft_collection(app.storage(), minter_address.clone());
    assert_eq!(collection.denom.clone().unwrap().id, collection_id);
    assert_eq!(
        collection.onfts.clone().into_iter().next().unwrap().id,
        token_id
    );
    // Query user minting details
    let user_minting_details: UserDetails = app
        .wrap()
        .query_wasm_smart(
            &minter_address,
            &MinterQueryMsg::UserMintingDetails {
                address: collector.clone().into_string(),
            },
        )
        .unwrap();
    assert_eq!(user_minting_details.total_minted_count, 1);
    assert_eq!(user_minting_details.minted_tokens[0].token_id, token_id);
    assert_eq!(user_minting_details.public_mint_count, 0);

    // At first round only one address can mint and per address limit is 1
    // Try minting once again with same address
    let error = app
        .execute_contract(
            collector.clone(),
            Addr::unchecked(minter_address.clone()),
            &MinterExecuteMsg::Mint {},
            &[round_1_price.clone()],
        )
        .unwrap_err();
    let res = error.source().unwrap().source().unwrap();
    let error = res.downcast_ref::<RoundWhitelistContractError>().unwrap();
    assert_eq!(
        error,
        &RoundWhitelistContractError::RoundReachedMintLimit {}
    );

    // Set block between round 1 and 2
    app.set_block(BlockInfo {
        chain_id: "test_1".to_string(),
        height: 1_000,
        time: Timestamp::from_nanos(3_000 + 1),
    });

    // Try minting with collector address
    let error = app
        .execute_contract(
            collector.clone(),
            Addr::unchecked(minter_address.clone()),
            &MinterExecuteMsg::Mint {},
            &[coin(1000000, "diffirent_denom")],
        )
        .unwrap_err();
    let res = error.source().unwrap();
    let error = res.downcast_ref::<MinterContractError>().unwrap();
    assert_eq!(error, &MinterContractError::WhitelistNotActive {});

    // Set block to round 2
    app.set_block(BlockInfo {
        chain_id: "test_1".to_string(),
        height: 1_000,
        time: Timestamp::from_nanos(4_000 + 1),
    });

    // Try minting with collector address
    let error = app
        .execute_contract(
            collector.clone(),
            Addr::unchecked(minter_address.clone()),
            &MinterExecuteMsg::Mint {},
            &[round_2_price.clone()],
        )
        .unwrap_err();
    let res = error.source().unwrap();
    let error = res.downcast_ref::<MinterContractError>().unwrap();
    assert_eq!(error, &MinterContractError::AddressNotWhitelisted {});

    // Try minting with creator address
    let res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(minter_address.clone()),
            &MinterExecuteMsg::Mint {},
            &[round_2_price.clone()],
        )
        .unwrap();
    let token_id: String = res.events[1].attributes[2].value.clone();
    let _collection_id: String = res.events[1].attributes[3].value.clone();
    // We are quering collection to check if it is minted from our mocked onft keeper
    let collection = query_onft_collection(app.storage(), minter_address.clone());
    assert_eq!(collection.onfts.clone()[1].id, token_id);

    // Query user minting details
    let user_minting_details: UserDetails = app
        .wrap()
        .query_wasm_smart(
            &minter_address,
            &MinterQueryMsg::UserMintingDetails {
                address: creator.clone().into_string(),
            },
        )
        .unwrap();
    assert_eq!(user_minting_details.total_minted_count, 1);
    assert_eq!(user_minting_details.minted_tokens[0].token_id, token_id);
    assert_eq!(user_minting_details.public_mint_count, 0);
}
