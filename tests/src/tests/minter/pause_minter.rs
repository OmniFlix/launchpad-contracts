use cosmwasm_std::Addr;
use cosmwasm_std::{coin, BlockInfo};
use cw_multi_test::Executor;

use minter_types::msg::QueryMsg;

use omniflix_minter_factory::msg::ExecuteMsg as FactoryExecuteMsg;
use pauser::PauseError;

use crate::helpers::mock_messages::factory_mock_messages::return_minter_factory_inst_message;
use crate::helpers::mock_messages::minter_mock_messages::return_minter_instantiate_msg;
use crate::helpers::utils::get_contract_address_from_res;

use crate::helpers::setup::setup;
use omniflix_minter::error::ContractError as MinterContractError;
use omniflix_minter::msg::ExecuteMsg as MinterExecuteMsg;
use omniflix_minter::msg::MinterExtensionQueryMsg;

type MinterQueryMsg = QueryMsg<MinterExtensionQueryMsg>;

#[test]
fn pause_minter() {
    let res = setup();
    let admin = res.test_accounts.admin;
    let creator = res.test_accounts.creator;
    let collector = res.test_accounts.collector;
    let minter_factory_code_id = res.minter_factory_code_id;
    let minter_code_id = res.minter_code_id;
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

    let minter_inst_msg = return_minter_instantiate_msg();
    let create_minter_msg = FactoryExecuteMsg::CreateMinter {
        msg: minter_inst_msg.clone(),
    };
    let public_minting_price = minter_inst_msg.init.mint_price;
    let public_start_time = minter_inst_msg.init.start_time;

    let res = app
        .execute_contract(
            admin.clone(),
            factory_addr,
            &create_minter_msg,
            &[coin(2000000, "uflix")],
        )
        .unwrap();
    let minter_address = get_contract_address_from_res(res.clone());

    // Ensure that the minter is not paused
    let is_paused: bool = app
        .wrap()
        .query_wasm_smart(&minter_address, &MinterQueryMsg::IsPaused {})
        .unwrap();
    assert_eq!(is_paused, false);

    // Set time to public minting time
    app.set_block(BlockInfo {
        time: public_start_time,
        height: 1,
        chain_id: "test".to_string(),
    });

    // Mint a token
    let mint_msg = MinterExecuteMsg::Mint {};

    let _res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(minter_address.clone()),
            &mint_msg,
            &[public_minting_price.clone()],
        )
        .unwrap();
    // Query pausers
    let pausers: Vec<Addr> = app
        .wrap()
        .query_wasm_smart(&minter_address, &MinterQueryMsg::Pausers {})
        .unwrap();
    assert_eq!(pausers.len(), 1);
    assert_eq!(pausers[0], creator);

    // Non pauser should not be able to pause
    let pause_msg = MinterExecuteMsg::Pause {};
    let res = app
        .execute_contract(
            collector.clone(),
            Addr::unchecked(minter_address.clone()),
            &pause_msg,
            &[],
        )
        .unwrap_err();
    let err = res.source().unwrap();
    let error = err.downcast_ref::<MinterContractError>().unwrap();
    assert_eq!(
        error,
        &MinterContractError::Pause(PauseError::Unauthorized {
            sender: collector.clone()
        })
    );

    // Pauser should be able to pause
    let pause_msg = MinterExecuteMsg::Pause {};
    let _res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(minter_address.clone()),
            &pause_msg,
            &[],
        )
        .unwrap();

    // Now query contract
    let is_paused: bool = app
        .wrap()
        .query_wasm_smart(&minter_address, &MinterQueryMsg::IsPaused {})
        .unwrap();
    assert!(is_paused);

    // Try pausing again
    let pause_msg = MinterExecuteMsg::Pause {};
    let res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(minter_address.clone()),
            &pause_msg,
            &[],
        )
        .unwrap_err();
    let err = res.source().unwrap();
    let error = err.downcast_ref::<MinterContractError>().unwrap();
    assert_eq!(error, &MinterContractError::Pause(PauseError::Paused {}));

    let mint_msg = MinterExecuteMsg::Mint {};
    let res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(minter_address.clone()),
            &mint_msg,
            &[coin(1000000, "uflix")],
        )
        .unwrap_err();
    let err = res.source().unwrap();
    let error = err.downcast_ref::<MinterContractError>().unwrap();
    assert_eq!(error, &MinterContractError::Pause(PauseError::Paused {}));

    // Try mint admin when paused
    let mint_msg = MinterExecuteMsg::MintAdmin {
        recipient: creator.clone().into_string(),
        token_id: None,
    };
    let res = app
        .execute_contract(
            creator.clone(),
            Addr::unchecked(minter_address.clone()),
            &mint_msg,
            &[coin(1000000, "uflix")],
        )
        .unwrap_err();
    let err = res.source().unwrap();
    let error = err.downcast_ref::<MinterContractError>().unwrap();
    assert_eq!(error, &MinterContractError::Pause(PauseError::Paused {}));
}
