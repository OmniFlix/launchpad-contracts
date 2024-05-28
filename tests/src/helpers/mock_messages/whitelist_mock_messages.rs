use cosmwasm_std::{Coin, Timestamp};

use whitelist_types::RoundConfig;

pub fn return_round_configs() -> Vec<RoundConfig> {
    let round_1 = whitelist_types::Round {
        start_time: Timestamp::from_nanos(2000),
        end_time: Timestamp::from_nanos(3000),
        mint_price: Coin::new(1000000, "diffirent_denom"),
        round_per_address_limit: 1,
    };
    let round_2 = whitelist_types::Round {
        start_time: Timestamp::from_nanos(4000),
        end_time: Timestamp::from_nanos(5000),
        mint_price: Coin::new(1000000, "uflix"),
        round_per_address_limit: 1,
    };
    let round_config_1 = whitelist_types::RoundConfig {
        round: round_1,
        members: vec!["collector".to_string()],
    };

    let round_config_2 = whitelist_types::RoundConfig {
        round: round_2,
        members: vec!["creator".to_string()],
    };

    vec![round_config_1, round_config_2]
}
