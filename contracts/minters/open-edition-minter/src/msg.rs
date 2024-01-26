use cosmwasm_schema::cw_serde;
use cosmwasm_std::Coin;

#[cw_serde]
pub enum ExecuteMsg {
    Mint {},
    MintAdmin { recipient: String },
    UpdateRoyaltyRatio { ratio: String },
    UpdateMintPrice { mint_price: Coin },
    UpdateWhitelistAddress { address: String },
    Pause {},
    Unpause {},
    SetPausers { pausers: Vec<String> },
}
