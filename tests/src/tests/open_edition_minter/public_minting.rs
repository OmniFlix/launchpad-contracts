#![cfg(test)]
use cosmwasm_std::{coin, Addr, BlockInfo, Timestamp, Uint128};

use crate::helpers::utils::{get_contract_address_from_res, mint_to_address};
use cw_multi_test::Executor;
use cw_utils::PaymentError;
use minter_types::types::UserDetails;
use omniflix_open_edition_minter_factory::msg::ExecuteMsg as OpenEditionMinterFactoryExecuteMsg;

use crate::helpers::mock_messages::factory_mock_messages::return_open_edition_minter_factory_inst_message;

use crate::helpers::mock_messages::oem_mock_messages::return_open_edition_minter_inst_msg;

use crate::helpers::utils::query_onft_collection;

use crate::helpers::setup::setup;
use omniflix_open_edition_minter::msg::OEMQueryExtension;

use minter_types::msg::QueryMsg as BaseMinterQueryMsg;

use omniflix_open_edition_minter::msg::ExecuteMsg as OpenEditionMinterExecuteMsg;

use omniflix_open_edition_minter::error::ContractError as OpenEditionMinterError;
type OpenEditionMinterQueryMsg = BaseMinterQueryMsg<OEMQueryExtension>;

#[test]
fn public_minting() {
    let res = setup();
    let admin = res.test_accounts.admin;
    let creator = res.test_accounts.creator;
    let collector = res.test_accounts.collector;
    let open_edition_minter_factory_code_id = res.open_edition_minter_factory_code_id;
    let open_edition_minter_code_id = res.open_edition_minter_code_id;
    let mut app = res.app;

    // Instantiate the oem minter factory
    let open_edition_minter_factory_instantiate_msg =
        return_open_edition_minter_factory_inst_message(open_edition_minter_code_id, None);

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

    // Create an open edition minter
    let open_edition_minter_instantiate_msg = return_open_edition_minter_inst_msg();
    let create_minter_msg = OpenEditionMinterFactoryExecuteMsg::CreateOpenEditionMinter {
        msg: open_edition_minter_instantiate_msg.clone(),
    };
    let public_minting_start_time = open_edition_minter_instantiate_msg.init.start_time;
    let public_minting_end_time = open_edition_minter_instantiate_msg.init.end_time.unwrap();
    let public_minting_price = open_edition_minter_instantiate_msg.init.mint_price.clone();

    let res = app
        .execute_contract(
            creator.clone(),
            open_edition_minter_factory_address.clone(),
            &create_minter_msg,
            &[coin(2000000, "uflix")],
        )
        .unwrap();
    let minter_address = get_contract_address_from_res(res);

    // Test queries before minting
    let query_msg = OpenEditionMinterQueryMsg::Extension(OEMQueryExtension::TokensRemaining {});
    let res: u32 = app
        .wrap()
        .query_wasm_smart(Addr::unchecked(minter_address.clone()), &query_msg)
        .unwrap();
    assert_eq!(res, 1000);

    let query_msg = OpenEditionMinterQueryMsg::TotalMintedCount {};
    let res: u32 = app
        .wrap()
        .query_wasm_smart(Addr::unchecked(minter_address.clone()), &query_msg)
        .unwrap();
    assert_eq!(res, 0);

    let query_msg = OpenEditionMinterQueryMsg::UserMintingDetails {
        address: collector.to_string(),
    };
    let res: UserDetails = app
        .wrap()
        .query_wasm_smart(Addr::unchecked(minter_address.clone()), &query_msg)
        .unwrap();
    assert_eq!(res.total_minted_count, 0);

    // Try Mint before public minting time
    app.set_block(BlockInfo {
        time: Timestamp::from_nanos(public_minting_start_time.nanos() - 1),
        height: 1,
        chain_id: "".to_string(),
    });
    let mint_msg = OpenEditionMinterExecuteMsg::Mint {};
    let res = app
        .execute_contract(
            collector.clone(),
            Addr::unchecked(minter_address.clone()),
            &mint_msg,
            &[public_minting_price.clone()],
        )
        .unwrap_err();
    let err = res.source().unwrap();
    let error = err.downcast_ref::<OpenEditionMinterError>().unwrap();
    assert_eq!(
        error,
        &OpenEditionMinterError::MintingNotStarted {
            start_time: public_minting_start_time,
            current_time: Timestamp::from_nanos(public_minting_start_time.nanos() - 1)
        }
    );

    // Try MintAdmin with non admin
    // Should fail
    let mint_admin_msg = OpenEditionMinterExecuteMsg::MintAdmin {
        recipient: collector.to_string(),
    };
    let res = app
        .execute_contract(
            collector.clone(),
            Addr::unchecked(minter_address.clone()),
            &mint_admin_msg,
            &[],
        )
        .unwrap_err();
    let err = res.source().unwrap();
    let error = err.downcast_ref::<OpenEditionMinterError>().unwrap();
    assert_eq!(error, &OpenEditionMinterError::Unauthorized {});

    // MintAdmin with admin
    // Admin is not affected by per address limit nor public minting start time
    // But sending payment is not permitted
    let mint_admin_msg = OpenEditionMinterExecuteMsg::MintAdmin {
        recipient: collector.to_string(),
    };
    let res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(minter_address.clone()),
            &mint_admin_msg,
            &[public_minting_price.clone()],
        )
        .unwrap_err();
    let err = res.source().unwrap();
    let error = err.downcast_ref::<OpenEditionMinterError>().unwrap();
    assert_eq!(
        error,
        &OpenEditionMinterError::PaymentError(PaymentError::NonPayable {})
    );

    // MintAdmin with admin
    let mint_admin_msg = OpenEditionMinterExecuteMsg::MintAdmin {
        recipient: collector.to_string(),
    };
    let _res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(minter_address.clone()),
            &mint_admin_msg,
            &[],
        )
        .unwrap();
    // Query collection
    let collection = query_onft_collection(app.storage(), minter_address.clone());
    assert_eq!(collection.onfts.clone()[0].id, "1");
    // Quert user minting details
    let query_msg = OpenEditionMinterQueryMsg::UserMintingDetails {
        address: collector.to_string(),
    };
    let res: UserDetails = app
        .wrap()
        .query_wasm_smart(Addr::unchecked(minter_address.clone()), &query_msg)
        .unwrap();
    assert_eq!(res.total_minted_count, 1);
    assert_eq!(res.minted_tokens[0].token_id, "1");
    // When minting with MintAdmin, the public mint count should not increase
    assert_eq!(res.public_mint_count, 0);

    // Query minter
    let query_msg = OpenEditionMinterQueryMsg::Extension(OEMQueryExtension::TokensRemaining {});
    let res: u32 = app
        .wrap()
        .query_wasm_smart(Addr::unchecked(minter_address.clone()), &query_msg)
        .unwrap();
    assert_eq!(res, 999);

    // Query minter
    let query_msg = OpenEditionMinterQueryMsg::TotalMintedCount {};
    let res: u32 = app
        .wrap()
        .query_wasm_smart(Addr::unchecked(minter_address.clone()), &query_msg)
        .unwrap();
    assert_eq!(res, 1);

    // Set block to public minting time
    app.set_block(BlockInfo {
        time: public_minting_start_time,
        height: 1,
        chain_id: "".to_string(),
    });

    // Try minting with incorrect payment amount
    let mint_msg = OpenEditionMinterExecuteMsg::Mint {};
    let res = app
        .execute_contract(
            collector.clone(),
            Addr::unchecked(minter_address.clone()),
            &mint_msg,
            &[coin(
                public_minting_price.amount.u128() - 1,
                public_minting_price.denom.clone(),
            )],
        )
        .unwrap_err();
    let err = res.source().unwrap();
    let error = err.downcast_ref::<OpenEditionMinterError>().unwrap();
    assert_eq!(
        error,
        &OpenEditionMinterError::IncorrectPaymentAmount {
            expected: public_minting_price.amount,
            sent: Uint128::from(public_minting_price.amount.u128() - 1)
        }
    );

    // Try minting with incorrect payment denom
    let mint_msg = OpenEditionMinterExecuteMsg::Mint {};
    let res = app
        .execute_contract(
            collector.clone(),
            Addr::unchecked(minter_address.clone()),
            &mint_msg,
            &[coin(public_minting_price.amount.u128(), "incorrect_denom")],
        )
        .unwrap_err();
    let err = res.source().unwrap();
    let error = err.downcast_ref::<OpenEditionMinterError>().unwrap();
    assert_eq!(
        error,
        &OpenEditionMinterError::PaymentError(PaymentError::ExtraDenom(
            "incorrect_denom".to_string()
        ))
    );
    // Check creator balance before mint
    let creator_balance_before_mint: Uint128 = app
        .wrap()
        .query_balance(creator.to_string(), public_minting_price.clone().denom)
        .unwrap()
        .amount;

    // Mint with collector
    let mint_msg = OpenEditionMinterExecuteMsg::Mint {};
    let _res = app
        .execute_contract(
            collector.clone(),
            Addr::unchecked(minter_address.clone()),
            &mint_msg,
            &[public_minting_price.clone()],
        )
        .unwrap();
    // Check creator balance after mint
    let creator_balance_after_mint: Uint128 = app
        .wrap()
        .query_balance(creator.to_string(), public_minting_price.clone().denom)
        .unwrap()
        .amount;
    // Check if creator got paid
    assert_eq!(
        creator_balance_after_mint,
        creator_balance_before_mint + public_minting_price.amount
    );
    // Query collection
    let collection = query_onft_collection(app.storage(), minter_address.clone());
    assert_eq!(collection.onfts.clone()[1].id, "2");
    assert_eq!(
        collection.onfts.clone()[1].metadata.clone().unwrap().name,
        "token_name #2".to_string()
    );
    assert_eq!(collection.onfts.clone()[1].owner, collector.to_string());
    // Query user minting details
    let query_msg = OpenEditionMinterQueryMsg::UserMintingDetails {
        address: collector.to_string(),
    };
    let res: UserDetails = app
        .wrap()
        .query_wasm_smart(Addr::unchecked(minter_address.clone()), &query_msg)
        .unwrap();
    assert_eq!(res.total_minted_count, 2);
    assert_eq!(res.minted_tokens[0].token_id, "1");
    assert_eq!(res.minted_tokens[1].token_id, "2");
    assert_eq!(res.public_mint_count, 1);

    // Query minter
    let query_msg = OpenEditionMinterQueryMsg::Extension(OEMQueryExtension::TokensRemaining {});
    let res: u32 = app
        .wrap()
        .query_wasm_smart(Addr::unchecked(minter_address.clone()), &query_msg)
        .unwrap();
    assert_eq!(res, 998);

    // Query minter
    let query_msg = OpenEditionMinterQueryMsg::TotalMintedCount {};
    let res: u32 = app
        .wrap()
        .query_wasm_smart(Addr::unchecked(minter_address.clone()), &query_msg)
        .unwrap();
    assert_eq!(res, 2);

    // Now mint once more with collector, Should fail as per address limit is 1
    let mint_msg = OpenEditionMinterExecuteMsg::Mint {};
    let res = app
        .execute_contract(
            collector.clone(),
            Addr::unchecked(minter_address.clone()),
            &mint_msg,
            &[public_minting_price.clone()],
        )
        .unwrap_err();

    let err = res.source().unwrap();
    let error = err.downcast_ref::<OpenEditionMinterError>().unwrap();
    assert_eq!(error, &OpenEditionMinterError::AddressReachedMintLimit {});

    // Set time to mint end time
    app.set_block(BlockInfo {
        time: Timestamp::from_nanos(public_minting_end_time.nanos() + 1),
        height: 1,
        chain_id: "".to_string(),
    });

    // Try minting after public minting end time
    // Nor admin or collector should be able to mint
    let mint_msg = OpenEditionMinterExecuteMsg::Mint {};
    let res = app
        .execute_contract(
            collector.clone(),
            Addr::unchecked(minter_address.clone()),
            &mint_msg,
            &[public_minting_price.clone()],
        )
        .unwrap_err();
    let err = res.source().unwrap();
    let error = err.downcast_ref::<OpenEditionMinterError>().unwrap();
    assert_eq!(error, &OpenEditionMinterError::PublicMintingEnded {});

    // MintAdmin with admin
    let mint_admin_msg = OpenEditionMinterExecuteMsg::MintAdmin {
        recipient: collector.to_string(),
    };
    let res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(minter_address.clone()),
            &mint_admin_msg,
            &[],
        )
        .unwrap_err();
    let err = res.source().unwrap();
    let error = err.downcast_ref::<OpenEditionMinterError>().unwrap();
    assert_eq!(error, &OpenEditionMinterError::PublicMintingEnded {});

    // Reset block time to public minting start time
    app.set_block(BlockInfo {
        time: public_minting_start_time,
        height: 1,
        chain_id: "".to_string(),
    });
    // Mint every remaining token to collectors
    for i in 3..1001 {
        let collector = Addr::unchecked(format!("collector{}", i));
        // Mint money for collector
        mint_to_address(
            &mut app,
            collector.clone().into_string(),
            [public_minting_price.clone()].to_vec(),
        );
        // Mint
        let mint_msg = OpenEditionMinterExecuteMsg::Mint {};
        let _res = app
            .execute_contract(
                collector.clone(),
                Addr::unchecked(minter_address.clone()),
                &mint_msg,
                &[public_minting_price.clone()],
            )
            .unwrap();
    }
    // Query minter
    let query_msg = OpenEditionMinterQueryMsg::Extension(OEMQueryExtension::TokensRemaining {});
    let res: u32 = app
        .wrap()
        .query_wasm_smart(Addr::unchecked(minter_address.clone()), &query_msg)
        .unwrap();
    assert_eq!(res, 0);

    // Query minter
    let query_msg = OpenEditionMinterQueryMsg::TotalMintedCount {};
    let res: u32 = app
        .wrap()
        .query_wasm_smart(Addr::unchecked(minter_address.clone()), &query_msg)
        .unwrap();
    assert_eq!(res, 1000);

    // Try minting after all tokens are minted
    let mint_msg = OpenEditionMinterExecuteMsg::Mint {};
    let res = app
        .execute_contract(
            collector.clone(),
            Addr::unchecked(minter_address.clone()),
            &mint_msg,
            &[public_minting_price.clone()],
        )
        .unwrap_err();
    let err = res.source().unwrap();
    let error = err.downcast_ref::<OpenEditionMinterError>().unwrap();
    assert_eq!(error, &OpenEditionMinterError::NoTokensLeftToMint {});

    // Try MintAdmin after all tokens are minted
    let mint_admin_msg = OpenEditionMinterExecuteMsg::MintAdmin {
        recipient: collector.to_string(),
    };
    let res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(minter_address.clone()),
            &mint_admin_msg,
            &[],
        )
        .unwrap_err();
    let err = res.source().unwrap();
    let error = err.downcast_ref::<OpenEditionMinterError>().unwrap();
    assert_eq!(error, &OpenEditionMinterError::NoTokensLeftToMint {});
}
