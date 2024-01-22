#[cfg(test)]
mod test_open_edition_minter_creation {

    use cosmwasm_std::{
        coin, to_json_binary, Addr, BlockInfo, Decimal, QueryRequest, StdError, Timestamp, Uint128,
        WasmQuery,
    };

    use cw_multi_test::Executor;
    use omniflix_open_edition_minter_factory::msg::{
        ExecuteMsg as OpenEditionMinterFactoryExecuteMsg,
        InstantiateMsg as OpenEditionMinterFactoryInstantiateMsg,
    };
    use open_edition_minter_types::InstantiateMsg as OpenEditionMinterInstantiateMsg;

    use whitelist_types::{Round, RoundWhitelistQueryMsgs};

    use crate::utils::{get_minter_address_from_res, return_minter_instantiate_msg, return_rounds};

    use crate::{setup::setup, utils::query_onft_collection};

    use omniflix_open_edition_minter::msg::ExecuteMsg as OpenEditionMinterExecuteMsg;
    use omniflix_round_whitelist::error::ContractError as RoundWhitelistContractError;
    use omniflix_round_whitelist_factory::error::ContractError as RoundWhitelistFactoryContractError;

    #[test]
    fn test_open_edition_minter_creation() {
        let (
            mut app,
            test_addresses,
            _minter_factory_code_id,
            _minter_code_id,
            _round_whitelist_factory_code_id,
            _round_whitelist_code_id,
            open_edition_minter_factory_code_id,
            open_edition_minter_code_id,
        ) = setup();
        let admin = test_addresses.admin;
        let creator = test_addresses.creator;
        let collector = test_addresses.collector;

        // Instantiate the minter factory
        let open_edition_minter_factory_instantiate_msg = OpenEditionMinterFactoryInstantiateMsg {
            admin: Some(admin.to_string()),
            allowed_minter_mint_denoms: vec!["uflix".to_string()],
            open_edition_minter_code_id,
            fee_collector_address: collector.to_string(),
            minter_creation_fee: coin(1000000, "uflix"),
        };

        let minter_factory_address = app
            .instantiate_contract(
                open_edition_minter_factory_code_id,
                admin.clone(),
                &open_edition_minter_factory_instantiate_msg,
                &[],
                "Open Edition Minter Factory",
                None,
            )
            .unwrap();
    }
}
