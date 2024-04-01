use cosmwasm_std::{Addr, Coin};
use omniflix_minter_factory::msg::{
    InstantiateMsg as MinterFactoryInstantiateMsg, MinterFactoryParams,
};
use omniflix_open_edition_minter_factory::msg::{
    InstantiateMsg as OpenEditionMinterFactoryInstantiateMsg, MultiMinterParams,
    OpenEditionMinterFactoryParams,
};
use omniflix_round_whitelist_factory::msg::{
    InstantiateMsg as RoundWhitelistFactoryInstantiateMsg, RoundWhitelistFactoryParams,
};

pub fn return_minter_factory_inst_message(code_id: u64) -> MinterFactoryInstantiateMsg {
    MinterFactoryInstantiateMsg {
        params: MinterFactoryParams {
            minter_code_id: code_id,
            minter_creation_fee: Coin::new(1000000, "uflix"),
            fee_collector_address: Addr::unchecked("admin".to_string()),
            admin: Addr::unchecked("admin".to_string()),
            product_label: "label".to_string(),
        },
    }
}

pub fn return_open_edition_minter_factory_inst_message(
    oem_code_id: u64,
    multi_mint_oem_code_id: Option<u64>,
) -> OpenEditionMinterFactoryInstantiateMsg {
    match multi_mint_oem_code_id {
        Some(multi_mint_oem_code_id) => OpenEditionMinterFactoryInstantiateMsg {
            params: OpenEditionMinterFactoryParams {
                open_edition_minter_code_id: oem_code_id,
                open_edition_minter_creation_fee: Coin::new(1000000, "uflix"),
                fee_collector_address: Addr::unchecked("admin".to_string()),
                admin: Addr::unchecked("admin".to_string()),
                multi_minter_params: Some(MultiMinterParams {
                    multi_minter_code_id: multi_mint_oem_code_id,
                    multi_minter_creation_fee: Coin::new(1000000, "uflix"),
                    multi_minter_product_label: "mm_oem_label".to_string(),
                }),
                oem_product_label: "oem_label".to_string(),
            },
        },
        None => OpenEditionMinterFactoryInstantiateMsg {
            params: OpenEditionMinterFactoryParams {
                open_edition_minter_code_id: oem_code_id,
                open_edition_minter_creation_fee: Coin::new(1000000, "uflix"),
                fee_collector_address: Addr::unchecked("admin".to_string()),
                admin: Addr::unchecked("admin".to_string()),
                multi_minter_params: None,
                oem_product_label: "oem_label".to_string(),
            },
        },
    }
}

pub fn return_round_whitelist_factory_inst_message(
    code_id: u64,
) -> RoundWhitelistFactoryInstantiateMsg {
    RoundWhitelistFactoryInstantiateMsg {
        params: RoundWhitelistFactoryParams {
            whitelist_code_id: code_id,
            whitelist_creation_fee: Coin::new(1000000, "uflix"),
            fee_collector_address: Addr::unchecked("admin".to_string()),
            admin: Addr::unchecked("admin".to_string()),
            product_label: "label".to_string(),
        },
    }
}
