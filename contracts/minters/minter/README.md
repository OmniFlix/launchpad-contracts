# Overview of the Classic Minter Contract

The Classic Minter contract is a core contract within the OmniFlix Hub, designed to facilitate the minting of unique NFTs.

This contract is managed by an `admin` who oversees the collection and its parameters. The contract supports the minting of NFTs that are associated with media stored on IPFS or similar decentralized storage networks.

The minting process can be initiated by any account interacting with the contract and can be set up to mint NFTs either for free or in exchange for a supported token.

Key features of the Classic Minter contract include:

- **Admin Management**: The contract admin has control over the collection and can configure various parameters related to minting.
- **Decentralized Media Storage**: Supports storing media on IPFS or similar networks, ensuring the decentralized and immutable storage of NFT content.
- **Flexible Minting Options**: Allows for both free and paid minting, providing flexibility for creators to set up their collections according to their needs.
- **User Interaction**: Users can interact with the contract to mint new NFTs, making the process straightforward and accessible.

This contract ensures that creators have a robust and flexible tool to manage their NFT collections on the OmniFlix Hub blockchain and related apps.

#### Instantiate

- During instantiation, the creator should send Collection details along with trading information such as price, denomination, and trading start time.

### Mint

- There are two types of minting: `Mint{}` and `AdminMint{}`
- `Mint{}`: This option is for users who want to own the NFT, and they need to pay the active price at that time.
- `AdminMint{}`: As the name suggests, this option is specifically for admin to mint a token. Admins have the ability to determine the recipient and specify the token ID. If the ID is available, it will be minted. Admins are not subject to address limits or private mint checks, and this action does not require a payment.

#### BurnRemainingTokens

- We cannot technically burn tokens because burnable ones are the ones that are not minted yet. If executed by the `admin`, this minter will not mint any other token.

#### UpdateRoyaltyRatio

- This function allows the `admin` to update the royalty ratio for the NFTs. The ratio is a string of decimal number.

    - `ratio`: The ratio of the royalty.

#### UpdateMintPrice

- This function permits the `admin` to modify the mint price. This only affects the price of the public mint.

    - `mint_price`: The price of the token.

#### RandomizeList

- `admin` has the ability to randomize token list. It's only gated by the `admin` because this operation is costly. In the future, a small fee could be collected from whoever wants to randomize the list.

#### UpdateWhitelistAddress
- This feature enables the `admin` to designate a whitelist address. Once set, the provided address should correspond to a whitelist contract, and private minting should not be initiated.

    - `address`: The address of the whitelist contract.

#### Pause
- This function allows the `admin` to pause the minting process. When paused, no new tokens can be minted.

#### Unpause
- This function allows the `admin` to resume the minting process after it has been paused.

#### SetPausers
- This function allows the `admin` to set the pausers list. The pausers are the addresses that can pause and unpause the minting process. Full list of pausers should be sent at every update.

    - `pausers`: List of pausers.

#### UpdateRoyaltyReceivers
- This function allows the `admin` to update the list of royalty receivers. The list includes weighted addresses, where the weight determines the percentage of royalties received by each address.

    - `receivers`: List of weighted addresses.

#### UpdateDenom
- This function allows the `admin` to update the collection name, description, and preview URI associated with the ONFT collection.

    - `collection_name`: The name of the collection.
    - `description`: The description of the collection.
    - `preview_uri`: The URI for the preview image of the collection.

#### PurgeDenom
- This function allows the `admin` to purge the collection. In order to do this, the collection should not have any tokens minted.

#### UpdateAdmin
- This function allows the `admin` to update the admin address. The new admin address should be provided.

    - `admin`: The address of the new admin.

#### UpdatePaymentCollector
- This function allows the `admin` to update the payment collector address. The new payment collector address should be provided.

    - `payment_collector`: The address of the new payment collector.