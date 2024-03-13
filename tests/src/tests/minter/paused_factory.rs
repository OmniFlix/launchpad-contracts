#![cfg(test)]
use crate::helpers::mock_messages::factory_mock_messages::return_minter_factory_inst_message;
use crate::helpers::mock_messages::minter_mock_messages::return_minter_instantiate_msg;
use crate::helpers::setup::setup;
use cosmwasm_std::coin;
use cw_multi_test::Executor;
use omniflix_minter_factory::error::ContractError as MinterFactoryError;
use omniflix_minter_factory::msg::ExecuteMsg as FactoryExecuteMsg;
use omniflix_minter_factory::msg::QueryMsg as MinterFactoryQueryMsg;
use pauser::PauseError;

#[test]
fn test_minter_creation() {
    let res = setup();
    let admin = res.test_accounts.admin;
    let creator = res.test_accounts.creator;
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
    // Ensure that the factory is not paused
    let is_paused: bool = app
        .wrap()
        .query_wasm_smart(&factory_addr, &MinterFactoryQueryMsg::IsPaused {})
        .unwrap();
    assert_eq!(is_paused, false);

    let minter_inst_msg = return_minter_instantiate_msg();
    let create_minter_msg = FactoryExecuteMsg::CreateMinter {
        msg: minter_inst_msg,
    };

    let _res = app
        .execute_contract(
            creator.clone(),
            factory_addr.clone(),
            &create_minter_msg,
            &[coin(2000000, "uflix")],
        )
        .unwrap();

    // Pause factory
    let pause_msg = FactoryExecuteMsg::Pause {};
    let _res = app
        .execute_contract(admin.clone(), factory_addr.clone(), &pause_msg, &[])
        .unwrap();

    // Ensure that the factory is paused
    let is_paused: bool = app
        .wrap()
        .query_wasm_smart(&factory_addr, &MinterFactoryQueryMsg::IsPaused {})
        .unwrap();
    assert_eq!(is_paused, true);

    // Ensure that the minter cannot be created
    let minter_inst_msg = return_minter_instantiate_msg();
    let create_minter_msg = FactoryExecuteMsg::CreateMinter {
        msg: minter_inst_msg,
    };

    let error = app
        .execute_contract(
            creator.clone(),
            factory_addr.clone(),
            &create_minter_msg,
            &[coin(2000000, "uflix")],
        )
        .unwrap_err();
    let res = error.source().unwrap();
    let error = res.downcast_ref::<MinterFactoryError>().unwrap();
    assert_eq!(error, &MinterFactoryError::Pause(PauseError::Paused {}));
}
