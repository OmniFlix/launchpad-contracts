# Overview of the Multi Mint Open Edition Minter (MM-OEM) Contract

The Multi Mint Open Edition Minter (MM-OEM) contract is an advanced version of the Open Edition Minter (OEM) contract, designed to support multiple NFT mint instances within the same collection. This contract introduces the concept of **"mint_instances"**, providing greater flexibility and configurability in the minting process.

## Key Features of MM-OEM

- **Multiple Mint Instances**: Manage several NFT configurations, called **mint_instances**, within a single contract and collection.
- **Custom Parameters Per Mint Instance**: Each mint_instance can have unique attributes like price, supply, and access controls.
- **Predefined Metadata**: Ensures each mint_instance has consistent and pre-set metadata.
- **Public and Private Minting**: Allows both private (whitelisted) and public minting phases, configurable for each mint_instance.
- **Single Contract Management**: Streamlines operations by managing all mint_instances and their NFTs under one contract.

This contract is ideal for creators looking to release multiple NFT configurations within a single collection, leveraging the benefits of both private and public minting phases while maintaining control over minting parameters for each mint_instance.

---

## Mint Instance

A **mint instance** is a configuration for a specific set of tokens. It includes the following parameters:

- `token_details`: Token details such as name, description, and preview_uri.
- `price`: The price of the token.
- `start_time`: The start time of minting.
- `end_time`: The end time of minting.
- `num_tokens`: The maximum supply of tokens in this mint_instance.
- `per_address_limit`: The maximum number of tokens that can be minted by a single address.

---

## Contract Interactions

This section outlines how users and administrators interact with the MM-OEM contract. While some actions are part of the initial setup (e.g., instantiation), others are executable messages that modify or interact with the contract during its lifecycle.

### Instantiate

The instantiation process initializes the contract. This step sets up the NFT collection metadata and prepares the contract for future interactions. It is important to note that **instantiation is not an executable message**; it is a one-time action performed when deploying the contract.

**Details:**
- The creator must provide the collection's details during instantiation.
- After instantiation, the contract does not include any mint instances. The creator must initiate the first mint_instance before NFTs can be minted.

---

### CreateMintInstance

This function allows the `admin` to create a new mint_instance with the given parameters. Once created, the mint_instance is added to the collection and becomes the active mint_instance. However, previous mint_instances remain active for minting unless explicitly paused or removed.

**Details:**
- `token_details`: Includes the name, symbol, description, and preview URI of the new NFT series.
- `config`: Specifies the parameters for the mint_instance, such as price, supply, and timeframes.

---

### Mint

There are two types of minting actions:

1. **`Mint{}`**: Users mint NFTs by paying the specified price for the active mint_instance.
    - **Example Input**:
      ```json
      {
        "mint_instance_id": "1"
      }
      ```

2. **`AdminMint{}`**: Admins mint NFTs without payment or restrictions, optionally specifying the recipient.

---

### Administrative Functions

#### UpdateRoyaltyRatio
- This function allows the `admin` to update the royalty ratio for the NFTs.
    - `ratio`: The new royalty ratio (as a string representing a decimal number).
    - `mint_instance_id`: The id of the instance. OPTIONAL. If not provided, it updates the active instance.

#### UpdateMintPrice
- This function allows the `admin` to update the mint price for the specified mint_instance.
    - `mint_price`: The new price of the token.
    - `mint_instance_id`: The id of the mint_instance to update. OPTIONAL. If not provided, it updates the active mint_instance.

#### UpdateWhitelistAddress
- This function allows the `admin` to set or update the whitelist contract address for the specified mint_instance.
    - `address`: The address of the whitelist contract.
    - `mint_instance_id`: The id of the mint_instance to update. OPTIONAL. If not provided, it updates the active mint_instance.

#### Pause / Unpause
- **Pause**: Allows the `admin` to pause the minting process. When paused, no new tokens can be minted.
- **Unpause**: Allows the `admin` to resume minting after it has been paused.

#### SetPausers
- This function allows the `admin` to designate specific accounts as "pausers."
    - `pausers`: A list of pauser addresses.

#### RemoveMintInstance
- This function allows the `admin` to remove a mint_instance. This is only possible if no tokens have been minted for that instance.
    - Upon removal, the active mint_instance will switch to the previous one. Removed mint_instance IDs will not be reused.

#### UpdateRoyaltyReceivers
- Allows the `admin` to set or update the royalty receivers and their respective weights.
    - `receivers`: List of receiver addresses with corresponding weights.

#### UpdateDenom
- This function allows the `admin` to modify metadata for the NFT collection (denom).
    - `collection_name`: The name of the collection.
    - `description`: The description of the collection.
    - `preview_uri`: URI for the collection's preview image.

#### PurgeDenom
- This function permanently removes the collection (denom). Only possible if no NFTs have been minted in the collection.

#### UpdateAdmin
- Changes the admin address to a new one.
    - `admin`: The new admin's address.

#### UpdatePaymentCollector
- Updates the address that collects payments for minting.
    - `payment_collector`: The new payment collector's address.

---

## Terminology Reference

### Active Mint Instance
The most recently created mint_instance. Its ID is the highest and is automatically set as the active one.

### Whitelist Contract
An external contract that manages private access for specific minting phases, enabling controlled or early access to certain users.

### Denom
Short for **denomination**, representing the entire NFT collection managed by the contract. Includes metadata like name, description, and preview URI.

### Royalty Ratio
A percentage or proportion of secondary sales revenue allocated to the royalty receivers.

### Pausers
Designated accounts authorized to pause and resume minting operations.

### Royalty Receivers
Addresses that receive royalties from secondary sales, with assigned weights determining their share.

### Admin
The address with administrative control over the contract, including creating mint_instances and updating contract parameters.

---

This contract empowers creators with versatile minting tools, ensuring streamlined NFT drops, robust configurations, and dynamic management capabilities.
