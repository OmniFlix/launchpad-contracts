# Overview of the Multi Mint Open Edition Minter (MM-OEM) Contract

The Multi Mint Open Edition Minter (MM-OEM) contract is an advanced version of the Open Edition Minter (OEM) contract, designed to support multiple NFT mints within the same collection. This contract introduces the concept of "drops," allowing for greater flexibility and configurability in the minting process.

Key features of the MM-OEM contract include:

- **Configurable Drops**: Unlike typical OEM contracts that manage a single mint, the MM-OEM contract allows users to configure multiple NFT mints, known as "drops," within the same contract and collection. Each drop can have its own parameters, providing enhanced flexibility for creators.
- **Predefined Metadata**: Uses predefined metadata for each drop, ensuring consistency and simplifying the minting process.
- **Integration with Whitelisting Contracts**: MM-OEM contracts are compatible with whitelisting contracts, enabling private minting rounds before opening to the public. This feature supports controlled access and tiered release strategies.
- **Public and Private Minting Rounds**: Facilitates both private and public minting rounds for each drop, giving creators the ability to manage and sequence their NFT releases effectively.
- **Single Contract Management**: Manages multiple NFT mints under a single contract, streamlining the deployment and management processes.

This contract is ideal for creators looking to release multiple NFT drops within a single collection, leveraging the benefits of both private and public minting phases while maintaining control over the minting parameters for each drop.

#### Drop

Drop is a configuration for a set of tokens. It includes the following parameters:

- `token_details`: Token details such as name, description, and preview_uri.
- `price`: The price of the token.
- `start_time`: The start time of the trading.
- `end_time`: The end time of the trading.
- `num_tokens`: The maximum supply of the token.
- `per_address_limit` : The maximum number of tokens that can be minted by a single address.

#### Instantiate

Similar to the Minter contract, the creator is required to send Collection details. This process initiates the creation of an NFT collection. It's important to note that the contract begins without any drop, meaning neither the creator nor the user can mint an NFT without the creator initiating the first drop.

### NewDrop

- This function allows the creator to create a new drop with the given parameters. The creator can create multiple drops with different parameters. Active drop will be changed to the new drop. But the previous drop will be still active for minting.
    - `token_details`: New token details such as name, symbol, description, and preview_uri.
    - `config`: New configuration for the new drop. It includes the following parameters:
        - `price`: The price of the token.
        - `start_time`: The start time of the trading.
        - `end_time`: The end time of the trading.
        - `num_tokens`: The maximum supply of the token.
        - `per_address_limit` : The maximum number of tokens that can be minted by a single address.

### Mint

- There are two types of minting: `Mint{}` and `AdminMint{}`
- `Mint{}`: This option is for users who want to own the NFT, and they need to pay the active price at that time.
    - `drop_id`: The id of the drop to mint. OPTIONAL. If not provided, it will mint the active drop.
- `AdminMint{}`: As the name suggests, this option is specifically for admin to mint a token. Admins have the ability to determine the recipient. Admins are not subject to address limits or private mint checks, and this action does not require a payment.
    - `drop_id`: The id of the drop to mint. OPTIONAL. If not provided, it will mint the active drop.
    - `recipient`: The address of the recipient.
