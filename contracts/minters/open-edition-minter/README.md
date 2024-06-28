# Overview of the Open Edition Minter (OEM) Contract

The Open Edition Minter (OEM) contract is designed to facilitate the minting of a single NFT within a collection, using predefined metadata. This type of minter contract allows for a flexible and scalable approach to NFT minting, where multiple users can mint the same NFT under a unified contract.

Key features of the OEM contract include:

- **Single NFT Collection**: Mints a single type of NFT within a collection, ensuring uniformity in the minted tokens.
- **Predefined Metadata**: Uses predefined metadata for the NFT, simplifying the minting process and ensuring consistency.
- **Integration with Whitelisting Contracts**: OEM contracts can work in conjunction with whitelisting contracts, enabling private minting rounds before opening to the public. This feature allows for controlled access to the minting process, supporting exclusive and tiered releases.

This contract is ideal for scenarios where creators wish to offer a single, consistent NFT to a broad audience, leveraging the benefits of both private and public minting phases.

#### Instantiate

Similar with the Minter contract, the creator should send Collection details along with trading information such as price, denomination, and trading start time. Factory contract will create a new OEM contract and initialize it with the given parameters.

### Mint

- There are two types of minting: `Mint{}` and `AdminMint{}`
- `Mint{}`: This option is for users who want to own the NFT, and they need to pay the active price at that time.
- `AdminMint{}`: As the name suggests, this option is specifically for admin to mint a token. Admins have the ability to determine the recipient. Admins are not subject to address limits or private mint checks, and this action does not require a payment.
    - `recipient`: The address of the recipient.

### UpdateRoyaltyRatio

- This function allows the `admin` to update the royalty ratio for the NFTs. The ratio is a string of decimal number.

    - `ratio`: The ratio of the royalty.

### UpdateMintPrice

- This function allows the `admin` to update the mint price of the NFT.

    - `mint_price`: The price of the token.

#### UpdateWhitelistAddress
- This feature enables the `admin` to designate a whitelist address. Once set, the provided address should correspond to a whitelist contract, and private minting should not be initiated.

    - `address`: The address of the whitelist contract.

### Pause

- This function allows the `admin` to pause the minting process. When paused, no new tokens can be minted.

### Unpause

- This function allows the `admin` to resume the minting process after it has been paused.

### SetPausers

- This function allows the `admin` to set the pausers list. The pausers are the addresses that can pause and unpause the minting process. Full list of pausers should be sent at every update.

    - `pausers`: List of pausers.

### UpdateRoyaltyReceivers


- This function allows the `admin` to update the list of royalty receivers. The list includes weighted addresses, where the weight determines the percentage of royalties received by each address.

    - `receivers`: List of weighted addresses.

### UpdateDenom

- This function allows the `admin` to update the collection name, description, and preview URI associated with the NFT collection.

    - `collection_name`: The name of the collection.
    - `description`: The description of the collection.
    - `preview_uri`: The URI for the preview image of the collection.

### PurgeDenom

- This function allows the `admin` to purge the collection. In order to do this, the collection should not have any tokens minted.

### UpdateAdmin

- This function allows the `admin` to update the admin address. The new admin address should be provided.

    - `admin`: The address of the new admin.

### UpdatePaymentCollector

- This function allows the `admin` to update the payment collector address. The new payment collector address should be provided.

    - `payment_collector`: The address of the new payment collector.

### BurnRemainingTokens

- This function allows the `admin` to stop minting any new tokens.