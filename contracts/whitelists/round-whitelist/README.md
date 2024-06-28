# Overview of the Round Whitelist Contract

The Round Whitelist contract is designed to manage and maintain a list of addresses eligible to participate in the token minting process. This contract supports multiple rounds of minting, each with configurable start and end times, enabling private minting phases based on the creator’s configuration.

Key features of the Round Whitelist contract include:

- **Eligibility Management**: Maintains a list of addresses that are eligible to mint tokens, ensuring controlled access to the minting process.
- **Configurable Rounds**: Allows the creator to define multiple rounds of minting, each with its own start and end times, along with sets of addresses that are permitted to mint. This flexibility supports various release strategies, including exclusive private rounds.
- **Single Round Functionality**: If desired, the contract can be configured to operate with a single round, functioning as a traditional whitelist.
- **Time-bound Access**: Each round is time-bound, with specific start and end times set by the creator. This ensures that minting can only occur during the designated periods.

This contract provides a robust solution for managing access to the minting process, offering flexibility and control to creators who wish to implement tiered or phased minting strategies.

### Instantiate 

- When creating an instance, the creator should submit the rounds along with the creation fee for the whitelist, as specified by the factory contract. None of these rounds should have started yet and they should not overlap.

### Execute

#### AddRound
- Creator of the whitelist can add a new round to the whitelist. The new round should not overlap with any existing rounds and should not have stated yet.

#### RemoveRound
- Creator of the whitelist can remove a round from the whitelist. The round should not have started yet.

#### PrivateMint
- This function is for the minter contract to call. It checks if the buyer is in the whitelist and if the round is active. If both conditions are met, the sender can mint the token. The private mint details are stored in the contract. More than one minter contract can call this function. Same buyer with different minter contracts can mint without effecting each other.
