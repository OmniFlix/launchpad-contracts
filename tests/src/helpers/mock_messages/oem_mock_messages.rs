use cosmwasm_std::{Coin, Decimal, Timestamp};
use minter_types::{collection_details::CollectionDetails, token_details::TokenDetails};
use omniflix_open_edition_minter_factory::msg::{
    OpenEditionMinterCreateMsg, OpenEditionMinterInitExtention,
};

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
        migration_data: None,
        token_details: Some(token_details),
    }
}
