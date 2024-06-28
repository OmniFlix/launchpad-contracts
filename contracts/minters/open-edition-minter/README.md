# Overview of the Open Edition Minter (OEM) Contract

The Open Edition Minter (OEM) contract is designed to facilitate the minting of a single NFT within a collection, using predefined metadata. This type of minter contract allows for a flexible and scalable approach to NFT minting, where multiple users can mint the same NFT under a unified contract.

Key features of the OEM contract include:

- **Single NFT Collection**: Mints a single type of NFT within a collection, ensuring uniformity in the minted tokens.
- **Predefined Metadata**: Uses predefined metadata for the NFT, simplifying the minting process and ensuring consistency.
- **Integration with Whitelisting Contracts**: OEM contracts can work in conjunction with whitelisting contracts, enabling private minting rounds before opening to the public. This feature allows for controlled access to the minting process, supporting exclusive and tiered releases.
- **Public and Private Minting Rounds**: Facilitates both private and public minting rounds, providing creators with the ability to manage and sequence their NFT drops effectively.

This contract is ideal for scenarios where creators wish to offer a single, consistent NFT to a broad audience, leveraging the benefits of both private and public minting phases.

#### Instantiate

Similar with the Minter contract, the creator should send Collection details along with trading information such as price, denomination, and trading start time. Factory contract will create a new OEM contract and initialize it with the given parameters.

### Mint

- There are two types of minting: `Mint{}` and `AdminMint{}`
- `Mint{}`: This option is for users who want to own the NFT, and they need to pay the active price at that time.
- `AdminMint{}`: As the name suggests, this option is specifically for admin to mint a token. Admins have the ability to determine the recipient. Admins are not subject to address limits or private mint checks, and this action does not require a payment.
    - `recipient`: The address of the recipient.