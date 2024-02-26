## Multi-Minter

Multi minter contract is a Open edition minter contract with configurable token parameters called "drop".

#### Drop

Drop is a configuration for a set of tokens. It includes the following parameters:

- `token_details`: Token details such as name, symbol, description, and preview_uri.
- `price`: The price of the token.
- `start_time`: The start time of the trading.
- `end_time`: The end time of the trading.
- `num_tokens`: The maximum supply of the token.
- `per_address_limit` : The maximum number of tokens that can be minted by a single address.

#### Instantiate

Similar with the Minter contract, the creator should send Collection details along with trading information such as price, denomination, and trading start time. Factory contract will create a new MultiMinter contract and initialize it with the given parameters. Upon creation, First drop will be created with the given parameters.

### NewDrop

- This function allows the creator to create a new drop with the given parameters. The creator can create multiple drops with different parameters. Active drop will be changed to the new drop. But the previous drop will be still active for minting.
    - `new_token_details`: New token details such as name, symbol, description, and preview_uri.
    - `new_config`: New configuration for the new drop. It includes the following parameters:
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