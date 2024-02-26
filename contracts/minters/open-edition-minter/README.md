## Open Edition Minter

Open edition minter contract is a minter contract which mints one NFT with predefined metadata.

#### Instantiate

Similar with the Minter contract, the creator should send Collection details along with trading information such as price, denomination, and trading start time. Factory contract will create a new OEM contract and initialize it with the given parameters.

### Mint

- There are two types of minting: `Mint{}` and `AdminMint{}`
- `Mint{}`: This option is for users who want to own the NFT, and they need to pay the active price at that time.
- `AdminMint{}`: As the name suggests, this option is specifically for admin to mint a token. Admins have the ability to determine the recipient. Admins are not subject to address limits or private mint checks, and this action does not require a payment.
    - `recipient`: The address of the recipient.