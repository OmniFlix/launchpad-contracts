## Round Whitelist Factory
Simple factory contract. Stores params for whitelist creation and creates whitelists.

### Parameters
- `whitelist_creation_fee`: The fee to create a whitelist
- `fee_collector_address`: The address to send the fee to
- `whitelist_code_id`: The code id of the whitelist contract
- `admin`: The admin address of the factory contract if not sent its the creator of the factory

