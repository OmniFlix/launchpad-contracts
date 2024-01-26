use cosmwasm_schema::cw_serde;
use cosmwasm_std::Coin;

#[cw_serde]
pub enum ExecuteMsg {
    Mint {},
    MintAdmin {
        recipient: String,
        token_id: Option<String>,
    },
    BurnRemainingTokens {},
    UpdateRoyaltyRatio {
        ratio: String,
    },
    UpdateMintPrice {
        mint_price: Coin,
    },
    RandomizeList {},
    UpdateWhitelistAddress {
        address: String,
    },
    Pause {},
    Unpause {},
    // This directly updates the pausers list if the sender is one of the pausers
    // At every update full list of pausers should be sent
    SetPausers {
        pausers: Vec<String>,
    },
}
