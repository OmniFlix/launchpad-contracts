## Round Whitelist

The Round whitelist contract maintains a list of addresses that are eligible to participate in the token sale.

Each round of the whitelist has an end time and a starting time. The creator of the whitelist contract can specify these parameters for each round, allowing for multiple rounds of private minting. However, if the creator decides to send out only one round, the whitelist contract can still function as a traditional whitelist.

### Instantiate 

- When creating an instance, the creator should submit the rounds along with the creation fee for the whitelist, as specified by the factory contract. None of these rounds should have started yet and they should not overlap.

### Execute

#### AddRound
- Creator of the whitelist can add a new round to the whitelist. The new round should not overlap with any existing rounds and should not have stated yet.

#### RemoveRound
- Creator of the whitelist can remove a round from the whitelist. The round should not have started yet.

#### PrivatelyMint
- This function is for the minter contract to call. It checks if the buyer is in the whitelist and if the round is active. If both conditions are met, the sender can mint the token. The private mint details are stored in the contract. More than one minter contract can call this function. Same buyer with different minter contracts can mint without effecting each other.
