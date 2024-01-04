/**
* This file was automatically generated by @cosmwasm/ts-codegen@0.35.3.
* DO NOT MODIFY IT BY HAND. Instead, modify the source JSONSchema file,
* and run the @cosmwasm/ts-codegen generate command to regenerate this file.
*/

export type Uint128 = string;
export type Timestamp = Uint64;
export type Uint64 = string;
export interface InstantiateMsg {
  admin?: string | null;
  collection_details: CollectionDetails;
  mint_denom: string;
  mint_price: Uint128;
  payment_collector?: string | null;
  per_address_limit: number;
  royalty_ratio: string;
  start_time: Timestamp;
  end_time?: Timestamp | null;
  whitelist_address?: string | null;
}
export interface CollectionDetails {
  base_uri: string;
  data: string;
  description: string;
  extensible: boolean;
  id: string;
  name: string;
  nsfw: boolean;
  num_tokens: number;
  preview_uri: string;
  schema: string;
  symbol: string;
  uri: string;
  uri_hash: string;
}
export type ExecuteMsg = {
  mint: {};
} | {
  mint_admin: {
    denom_id?: string | null;
    recipient: string;
  };
} | {
  burn_remaining_tokens: {};
} | {
  update_royalty_ratio: {
    ratio: string;
  };
} | {
  update_mint_price: {
    mint_price: Uint128;
  };
} | {
  randomize_list: {};
};
export type QueryMsg = {
  collection: {};
} | {
  config: {};
} | {
  mintable_tokens: {};
} | {
  minted_tokens: {
    address: string;
  };
} | {
  total_tokens: {};
};
export type Addr = string;
export type Decimal = string;
export interface Config {
  admin: Addr;
  mint_price: Coin;
  payment_collector: Addr;
  per_address_limit: number;
  royalty_ratio: Decimal;
  start_time: Timestamp;
  whitelist_address?: Addr | null;
}
export interface Coin {
  amount: Uint128;
  denom: string;
  [k: string]: unknown;
}
export type ArrayOfToken = Token[];
export interface Token {
  token_id: string;
}
export interface UserDetails {
  minted_tokens: Token[];
  total_minted_count: number;
}
export type Uint32 = number;