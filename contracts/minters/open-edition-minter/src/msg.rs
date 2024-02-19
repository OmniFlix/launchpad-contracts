use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Coin, Uint128};
use omniflix_std::types::omniflix::onft::v1beta1::WeightedAddress;

#[cw_serde]
pub enum ExecuteMsg {
    Mint {},
    MintAdmin {
        recipient: String,
    },
    UpdateRoyaltyRatio {
        ratio: String,
    },
    UpdateMintPrice {
        mint_price: Coin,
    },
    UpdateWhitelistAddress {
        address: String,
    },
    Pause {},
    Unpause {},
    SetPausers {
        pausers: Vec<String>,
    },
    UpdateRoyaltyReceivers {
        receivers: Vec<WeightedAddress>,
    },
    UpdateDenom {
        name: Option<String>,
        description: Option<String>,
        preview_uri: Option<String>,
    },
    PurgeDenom {},
}

#[cw_serde]
pub enum OEMQueryExtension {
    TokensRemaining {},
}
