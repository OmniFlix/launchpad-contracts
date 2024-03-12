use cosmwasm_std::{Addr, Coin, Timestamp};

use whitelist_types::Round;

pub fn return_rounds() -> Vec<Round> {
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
