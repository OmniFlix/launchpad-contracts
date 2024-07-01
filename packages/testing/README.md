## OmniFlix Launchpad Testing
This package contains a custom testing app for the OmniFlix Launchpad contracts. It uses the multi-test package with a custom Stargate keeper.
### Stargate Keeper
The Stargate keeper is a custom keeper that is implemented for contracts Stargate queries and returns the result of the query. It is used for testing purposes. Additionally, a very primitive NFT keeper is implemented for this keeper.