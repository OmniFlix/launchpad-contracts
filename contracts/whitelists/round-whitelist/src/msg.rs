use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Coin, Timestamp};

use whitelist_types::Round;

#[cw_serde]
pub enum ExecuteMsg {
    RemoveRound {
        round_index: u32,
    },
    AddRound {
        round: Round,
    },
    PrivateMint {
        collector: String,
    },
    AddMembers {
        address: Vec<String>,
        round_index: u32,
    },
    UpdatePrice {
        mint_price: Coin,
        round_index: u32,
    },
}
