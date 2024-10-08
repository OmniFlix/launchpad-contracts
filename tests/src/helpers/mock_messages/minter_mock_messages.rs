use cosmwasm_std::{Addr, Coin, Decimal, Timestamp};
use minter_types::{
    collection_details::CollectionDetails, token_details::TokenDetails, types::AuthDetails,
};
use omniflix_minter_factory::msg::{CreateMinterMsg, MinterInitExtention};

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
        init: Some(MinterInitExtention {
            mint_price: Coin::new(1000000, "uflix"),
            start_time: Timestamp::from_nanos(1_000_000_000),
            end_time: Some(Timestamp::from_nanos(2_000_000_000)),
            per_address_limit: Some(1),
            whitelist_address: None,
            num_tokens: 50,
        }),
        auth_details: AuthDetails {
            admin: Addr::unchecked("creator".to_string()),
            payment_collector: Addr::unchecked("creator".to_string()),
        },
    }
}
