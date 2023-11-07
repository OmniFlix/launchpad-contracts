# Omniflix Launchpad Readme

## Omniflix Launchpad Contracts

This repository contains the smart contracts for the Omniflix Launchpad platform. These contracts are responsible for launching an NFT collection.

## Contracts

- Omniflix-whitelist
- Omniflix-minter

## Contract Overview

### Minter

The Minter contract is the primary component of the launchpad.

#### Instantiate

- During instantiation, the creator should send Collection details along with trading information such as price, denomination, and trading start time.
- The creator also has the option to send `Rounds`.
    - Currently, there are two types of rounds.
    - During instantiation, one of these types should be included inside an array.
    - The creator can add as many rounds as desired, but overlapping rounds are not permitted and will cause an error.

#### WhitelistAddress
``` 
{
  address: Addr,
  start_time: Option<Timestamp>,
  end_time: Option<Timestamp>,
  mint_price: Uint128,
  round_limit: u32,
}
```

#### WhitelistCollection
```
{
  collection_id: String,
  start_time: Timestamp,
  end_time: Timestamp,
  mint_price: Uint128,
  round_limit: u32,
}
```

- `Whitelist address`
    - This whitelist type refers to the whitelist contract in our launchpad. To use it, you need to instantiate it and send its address to the minter along with the other parameters.
    - The optional parts do not need to be sent. When instantiating, the minter contract ignores these parameters and queries the corresponding whitelist contract.
- `Whitelist collection`
    - This whitelist is used to allow specific collections that are already created and have token holders. The contract checks the specified collection to determine if the round is active and if the buyer actually possesses at least one of those tokens. If these conditions are met, the user will be granted permission for private minting.

#### Mint

- There are two types of minting: `Mint{}` and `AdminMint{}`
- `Mint{}`: This option is for users who want to own the NFT, and they need to pay the active price at that time.
- `AdminMint{}`: As the name suggests, this option is specifically for admin to mint a token. Admins have the ability to determine the recipient and specify the token ID. If the ID is available, it will be minted. Admins are not subject to address limits or private mint checks, and this action does not require a payment.

#### RemoveRound

#### AddRound

- Creator has the ability to add or remove rounds as they please. When adding overlapping rounds are still not permitted and rounds that already started cannot be added nor removed.

#### UpdateCollectionRound
- Creator has the ability to update the collection round. This is useful when the creator wants to change the collection ID, start time, end time, or mint price. This function is only available for collection rounds.

#### UpdateWhitelistRound
- This function is can not be used by the creator. It is only available for the whitelist contract to update the whitelist round. This is useful when the creator wants to change the start time, end time, or mint price. This changes happen on the whitelist contract. But the whitelist contract updates this round on the minter contract.

#### BurnRemainingTokens

- We cannot technically burn tokens because burnable ones are the ones that are not minted yet. If executed by the creator, this minter will not mint any other token.

#### UpdateRoyaltyRatio
- This function allows the creator to change the royalty ratio for the NFTs minted through the launchpad. The royalty ratio determines the percentage of each subsequent resale of an NFT that is paid to the original creator as royalties. 

#### UpdateMintPrice
- This function permits the creator to modify the mint price. This only affects the price of the public mint.

#### RandomizeList

- Creator has the ability to randomize token list. It's only gated by the creator because this operation is costly. In the future, a small fee could be collected from whoever wants to randomize the list.

### Whitelist

The Whitelist contract maintains a list of addresses that are eligible to participate in the token sale.

#### Instantiate

- Upon instantiation, the creator should send the start and end time of this whitelist along with the mint price and an array of valid addresses.

#### Execute

- `UpdateEndTime`: This message is used to update the end time of the whitelist. The `end_time` parameter should be provided along with an optional `minter_address` parameter.
- `UpdateMintPrice`: This message is used to update the mint price of the whitelist. The `mint_price` parameter should be provided along with an optional `minter_address` parameter.
- `UpdatePerAddressLimit`: This message is used to update the per-address limit of the whitelist. The `amount` parameter should be the new limit, and an optional `minter_address` parameter can be provided.
- `UpdateStartTime`: This message is used to change the start time of the whitelist. The `start_time` parameter should be provided along with an optional `minter_address` parameter.
    - If the creator has a specific minter contract that is already using this contract as one of its rounds, they are expected to provide the minter address.
    - In such cases, the parameters for each contract will be different, which may cause overlap. Therefore, if the minter address is provided, the whitelist contract will attempt to update the parameter on the minter contract. (Note: On the minter contract, the whitelist round can only be updated from the whitelist address.)
- `AddMembers`: This message is used to add new addresses to the whitelist. The `addresses` parameter should be an array of valid addresses.
- `RemoveMembers`: This message is used to remove addresses from the whitelist. The `addresses` parameter should be an array of valid addresses.
- `IncreaseMemberLimit`: This message is used to increase the member limit of the whitelist by a specified amount.
- `UpdateAdmin`: This message is used to update the admin of the whitelist. The `admin` parameter should be the new admin address.
- `Freeze`: This message is used to freeze the whitelist, preventing any further changes.

## Contributing

If you would like to contribute to the Omniflix Launchpad contracts, please follow the guidelines outlined in [CONTRIBUTING.md](http://contributing.md/). We welcome all contributions, from bug fixes to feature enhancements.