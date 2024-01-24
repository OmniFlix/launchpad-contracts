## Minter factory
Simple factory contract. Stores params for minter creation and creates minters.

### Parameters
- `minter_creation_fee`: The fee to create a minter
- `fee_collector_address`: The address to send the fee to
- `minter_code_id`: The code id of the minter contract
- `allowed_minter_mint_denoms`: The denoms that creator can set as minting price denom
- `admin`: The admin address of the minter contract if not sent its the creator of the factory



