use cosmwasm_std::{from_json, Addr, Coin, MemoryStorage, Storage, Timestamp, Uint128};
use cw_multi_test::AppResponse;
use minter_types::{CollectionDetails, InstantiateMsg as MinterInstantiateMsg};
use omniflix_std::types::omniflix::onft::v1beta1::Collection;

pub fn get_minter_address_from_res(res: AppResponse) -> String {
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

pub fn return_minter_instantiate_msg() -> MinterInstantiateMsg {
    let collection_details = CollectionDetails {
        name: "name".to_string(),
        description: "description".to_string(),
        preview_uri: "preview_uri".to_string(),
        schema: "schema".to_string(),
        symbol: "symbol".to_string(),
        id: "id".to_string(),
        extensible: true,
        nsfw: false,
        base_uri: "base_uri".to_string(),
        uri: "uri".to_string(),
        uri_hash: "uri_hash".to_string(),
        data: "data".to_string(),
    };

    MinterInstantiateMsg {
        per_address_limit: 1,
        admin: Some("creator".to_string()),
        collection_details,
        mint_denom: "uflix".to_string(),
        mint_price: Uint128::from(1000000u128),
        start_time: Timestamp::from_nanos(1_000_000_000),
        royalty_ratio: "0.1".to_string(),
        payment_collector: Some("payment_collector".to_string()),
        whitelist_address: None,
        end_time: Some(Timestamp::from_nanos(1_000_000_000 + 1_000_000_000)),
        num_tokens: 1000,
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
