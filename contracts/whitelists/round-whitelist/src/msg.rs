use cosmwasm_schema::cw_serde;
use cosmwasm_std::Coin;

use whitelist_types::RoundConfig;

#[cw_serde]
pub enum ExecuteMsg {
    RemoveRound {
        round_index: u8,
    },
    AddRound {
        round_config: RoundConfig,
    },
    PrivateMint {
        collector: String,
    },
    AddMembers {
        members: Vec<String>,
        round_index: u8,
    },
    UpdatePrice {
        mint_price: Coin,
        round_index: u8,
    },
}
