#[cfg(test)]

mod test_pause {

    use cosmwasm_std::{
        coin, coins, to_json_binary, Addr, BlockInfo, QueryRequest, Timestamp, Uint128, WasmQuery,
    };
    use cw_multi_test::{BankSudo, Executor, SudoMsg};
    use minter_types::PauseError;
    use minter_types::Token;
    use minter_types::UserDetails;

    use minter_types::QueryMsg;
    use omniflix_minter::msg::ExecuteMsg as MinterExecuteMsg;
    use omniflix_minter_factory::msg::{
        ExecuteMsg as FactoryExecuteMsg, InstantiateMsg as FactoryInstantiateMsg,
    };

    use crate::utils::{get_minter_address_from_res, return_minter_instantiate_msg, return_rounds};

    use crate::{setup::setup, utils::query_onft_collection};
    use omniflix_minter::error::ContractError as MinterContractError;

    use omniflix_round_whitelist::error::ContractError as RoundWhitelistContractError;

    #[test]
    fn test_pause() {
        let (
            mut app,
            test_addresses,
            minter_factory_code_id,
            minter_code_id,
            _round_whitelist_factory_code_id,
            _round_whitelist_code_id,
            _open_edition_minter_code_id,
            _open_edition_minter_factory_code_id,
        ) = setup();
        let admin = test_addresses.admin;
        let creator = test_addresses.creator;
        let collector = test_addresses.collector;

        let factory_inst_msg = FactoryInstantiateMsg {
            admin: Some(admin.to_string()),
            minter_creation_fee: coin(1000000, "uflix"),
            minter_code_id,
            fee_collector_address: admin.clone().into_string(),
            allowed_minter_mint_denoms: vec!["uflix".to_string()],
        };
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
            msg: minter_inst_msg,
        };
        let res = app
            .execute_contract(
                creator.clone(),
                factory_addr,
                &create_minter_msg,
                &[coin(2000000, "uflix")],
            )
            .unwrap();
        let minter_address = get_minter_address_from_res(res.clone());

        // Now query contract
        let is_paused: bool = app
            .wrap()
            .query(&QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr: minter_address.clone(),
                msg: to_json_binary(&QueryMsg::IsPaused {}).unwrap(),
            }))
            .unwrap();
        assert_eq!(is_paused, false);

        // Query pausers
        let pausers: Vec<Addr> = app
            .wrap()
            .query(&QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr: minter_address.clone(),
                msg: to_json_binary(&QueryMsg::Pausers {}).unwrap(),
            }))
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
        let res = app
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
            .query(&QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr: minter_address.clone(),
                msg: to_json_binary(&QueryMsg::IsPaused {}).unwrap(),
            }))
            .unwrap();
        assert_eq!(is_paused, true);

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

        // Try minting set block time to 1_000_000_000
        let block = BlockInfo {
            height: 1,
            time: Timestamp::from_nanos(1_000_000_000 + 1),
            chain_id: "cosmos-testnet".to_string(),
        };
        app.set_block(block);

        let mint_msg = MinterExecuteMsg::Mint {};
        let res = app
            .execute_contract(
                collector.clone(),
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
            recipient: collector.clone().to_string(),
            token_id: Some("token_id".to_string()),
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

        // unpause
        let unpause_msg = MinterExecuteMsg::Unpause {};
        let res = app
            .execute_contract(
                creator.clone(),
                Addr::unchecked(minter_address.clone()),
                &unpause_msg,
                &[],
            )
            .unwrap();

        // Now query contract
        let is_paused: bool = app
            .wrap()
            .query(&QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr: minter_address.clone(),
                msg: to_json_binary(&QueryMsg::IsPaused {}).unwrap(),
            }))
            .unwrap();
        assert_eq!(is_paused, false);

        // Try minting
        let mint_msg = MinterExecuteMsg::Mint {};
        let res = app
            .execute_contract(
                collector.clone(),
                Addr::unchecked(minter_address.clone()),
                &mint_msg,
                &[coin(1000000, "uflix")],
            )
            .unwrap();

        // Set pausers
        let set_pausers_msg = MinterExecuteMsg::SetPausers {
            pausers: vec![collector.clone().to_string()],
        };
        let res = app
            .execute_contract(
                creator.clone(),
                Addr::unchecked(minter_address.clone()),
                &set_pausers_msg,
                &[],
            )
            .unwrap();

        // Now query contract
        let pausers: Vec<Addr> = app
            .wrap()
            .query(&QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr: minter_address.clone(),
                msg: to_json_binary(&QueryMsg::Pausers {}).unwrap(),
            }))
            .unwrap();
        assert_eq!(pausers.len(), 1);
        assert_eq!(pausers[0], collector);

        // Try pausing again with creator
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
        assert_eq!(
            error,
            &MinterContractError::Pause(PauseError::Unauthorized {
                sender: creator.clone()
            })
        );

        // Try pausing again with collector
        let pause_msg = MinterExecuteMsg::Pause {};
        let res = app
            .execute_contract(
                collector.clone(),
                Addr::unchecked(minter_address.clone()),
                &pause_msg,
                &[],
            )
            .unwrap();

        // Now query contract
        let is_paused: bool = app
            .wrap()
            .query(&QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr: minter_address.clone(),
                msg: to_json_binary(&QueryMsg::IsPaused {}).unwrap(),
            }))
            .unwrap();
        assert_eq!(is_paused, true);
    }
}
