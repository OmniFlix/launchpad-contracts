use cosmwasm_std::{from_json, Addr, Coin, Decimal, MemoryStorage, Storage, Timestamp};
use cw_multi_test::{AppResponse, BankSudo, SudoMsg};
use minter_types::types::{CollectionDetails, TokenDetails};
use omniflix_minter_factory::msg::{CreateMinterMsg, MinterInitExtention};
use omniflix_minter_factory::msg::{
    InstantiateMsg as MinterFactoryInstantiateMsg, MinterFactoryParams,
};
use omniflix_open_edition_minter_factory::msg::{
    InstantiateMsg as OpenEditionMinterFactoryInstantiateMsg, MultiMinterParams,
    OpenEditionMinterCreateMsg, OpenEditionMinterFactoryParams, OpenEditionMinterInitExtention,
};
use omniflix_round_whitelist_factory::msg::{
    InstantiateMsg as RoundWhitelistFactoryInstantiateMsg, RoundWhitelistFactoryParams,
};
use omniflix_std::types::omniflix::onft::v1beta1::Collection;
use omniflix_testing::app::OmniflixApp;
pub fn get_contract_address_from_res(res: AppResponse) -> String {
    res.events
        .iter()
        .find(|e| e.ty == "instantiate")
        .unwrap()
        .attributes
        .iter()
        .find(|a| a.key == "_contract_address")
        .unwrap()
        .value
        .clone()
}

pub fn query_onft_collection(storage: &MemoryStorage, minter_address: String) -> Collection {
    let key = format!("collections:{}:{}", "collection", minter_address);
    let collection = storage.get(key.as_bytes()).unwrap();
    let collection_details: Collection = from_json(collection).unwrap();
    collection_details
}

pub fn return_minter_instantiate_msg() -> CreateMinterMsg {
    let collection_details = CollectionDetails {
        collection_name: "name".to_string(),
        description: Some("description".to_string()),
        preview_uri: Some("preview_uri".to_string()),
        schema: Some("schema".to_string()),
        symbol: "symbol".to_string(),
        id: "id".to_string(),
        uri_hash: Some("uri_hash".to_string()),
        data: Some("data".to_string()),
        royalty_receivers: None,
        uri: Some("uri".to_string()),
    };
    let token_details = TokenDetails {
        token_name: "token_name".to_string(),
        description: Some("description".to_string()),
        preview_uri: Some("preview_uri".to_string()),
        base_token_uri: "base_token_uri".to_string(),
        transferable: true,
        royalty_ratio: Decimal::percent(10),
        extensible: true,
        nsfw: false,
        data: None,
    };

    CreateMinterMsg {
        collection_details,
        token_details: Some(token_details),
        init: MinterInitExtention {
            admin: "creator".to_string(),
            mint_price: Coin::new(1000000, "uflix"),
            start_time: Timestamp::from_nanos(1_000_000_000),
            end_time: Some(Timestamp::from_nanos(2_000_000_000)),
            per_address_limit: Some(1),
            payment_collector: Some("creator".to_string()),
            whitelist_address: None,
            num_tokens: 1000,
        },
    }
}

pub fn return_open_edition_minter_inst_msg() -> OpenEditionMinterCreateMsg {
    let collection_details = CollectionDetails {
        collection_name: "name".to_string(),
        description: Some("description".to_string()),
        preview_uri: Some("preview_uri".to_string()),
        schema: Some("schema".to_string()),
        symbol: "symbol".to_string(),
        id: "id".to_string(),
        uri: Some("uri".to_string()),
        uri_hash: Some("uri_hash".to_string()),
        data: Some("data".to_string()),
        royalty_receivers: None,
    };
    let init = OpenEditionMinterInitExtention {
        admin: "creator".to_string(),
        mint_price: Coin::new(1000000, "uflix"),
        start_time: Timestamp::from_nanos(1_000_000_000),
        end_time: Some(Timestamp::from_nanos(2_000_000_000)),
        per_address_limit: Some(1),
        payment_collector: Some("creator".to_string()),
        whitelist_address: None,
        num_tokens: Some(1000),
    };
    let token_details = TokenDetails {
        token_name: "token_name".to_string(),
        description: Some("description".to_string()),
        preview_uri: Some("preview_uri".to_string()),
        base_token_uri: "base_token_uri".to_string(),
        transferable: true,
        royalty_ratio: Decimal::percent(10),
        extensible: true,
        nsfw: false,
        data: None,
    };

    OpenEditionMinterCreateMsg {
        collection_details,
        init,
        token_details: Some(token_details),
    }
}

pub fn return_rounds() -> Vec<whitelist_types::Round> {
    // Lets create 2 rounds
    let round_1 = whitelist_types::Round {
        start_time: Timestamp::from_nanos(2000),
        end_time: Timestamp::from_nanos(3000),
        addresses: vec![Addr::unchecked("collector".to_string())],
        mint_price: Coin::new(1000000, "diffirent_denom"),
        round_per_address_limit: 1,
    };
    let round_2 = whitelist_types::Round {
        start_time: Timestamp::from_nanos(4000),
        end_time: Timestamp::from_nanos(5000),
        addresses: vec![Addr::unchecked("creator".to_string())],
        mint_price: Coin::new(1000000, "uflix"),
        round_per_address_limit: 1,
    };
    let rounds = vec![round_1.clone(), round_2.clone()];

    rounds
}
pub fn mint_to_address(app: &mut OmniflixApp, to_address: String, amount: Vec<Coin>) {
    app.sudo(SudoMsg::Bank(BankSudo::Mint { to_address, amount }))
        .unwrap();
}
pub fn return_minter_factory_inst_message(code_id: u64) -> MinterFactoryInstantiateMsg {
    let msg = MinterFactoryInstantiateMsg {
        params: MinterFactoryParams {
            minter_code_id: code_id,
            minter_creation_fee: Coin::new(1000000, "uflix"),
            fee_collector_address: Addr::unchecked("admin".to_string()),
            admin: Addr::unchecked("admin".to_string()),
            product_label: "label".to_string(),
        },
    };
    msg
}

pub fn return_open_edition_minter_factory_inst_message(
    oem_code_id: u64,
    multi_mint_oem_code_id: u64,
) -> OpenEditionMinterFactoryInstantiateMsg {
    let msg = OpenEditionMinterFactoryInstantiateMsg {
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
    };
    msg
}

pub fn return_round_whitelist_factory_inst_message(
    code_id: u64,
) -> RoundWhitelistFactoryInstantiateMsg {
    let msg = RoundWhitelistFactoryInstantiateMsg {
        params: RoundWhitelistFactoryParams {
            whitelist_code_id: code_id,
            whitelist_creation_fee: Coin::new(1000000, "uflix"),
            fee_collector_address: Addr::unchecked("admin".to_string()),
            admin: Addr::unchecked("admin".to_string()),
            product_label: "label".to_string(),
        },
    };
    msg
}
