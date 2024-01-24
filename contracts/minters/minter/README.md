## Minter

The Minter contract is the primary component of the launchpad.

#### Instantiate

- During instantiation, the creator should send Collection details along with trading information such as price, denomination, and trading start time.

### Mint

- There are two types of minting: `Mint{}` and `AdminMint{}`
- `Mint{}`: This option is for users who want to own the NFT, and they need to pay the active price at that time.
- `AdminMint{}`: As the name suggests, this option is specifically for admin to mint a token. Admins have the ability to determine the recipient and specify the token ID. If the ID is available, it will be minted. Admins are not subject to address limits or private mint checks, and this action does not require a payment.

#### BurnRemainingTokens

- We cannot technically burn tokens because burnable ones are the ones that are not minted yet. If executed by the creator, this minter will not mint any other token.

#### UpdateRoyaltyRatio

- This function allows the creator to change the royalty ratio for the NFTs minted through the launchpad. The royalty ratio determines the percentage of each subsequent resale of an NFT that is paid to the original creator as royalties.

#### UpdateMintPrice

- This function permits the creator to modify the mint price. This only affects the price of the public mint.

#### RandomizeList

- Creator has the ability to randomize token list. It's only gated by the creator because this operation is costly. In the future, a small fee could be collected from whoever wants to randomize the list.