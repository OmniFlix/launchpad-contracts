/**
* This file was automatically generated by @cosmwasm/ts-codegen@0.35.3.
* DO NOT MODIFY IT BY HAND. Instead, modify the source JSONSchema file,
* and run the @cosmwasm/ts-codegen generate command to regenerate this file.
*/

export type Uint128 = string;
export interface InstantiateMsg {
  admin?: string | null;
  collection_details: CollectionDetails;
  mint_denom: string;
  mint_price: Uint128;
  payment_collector?: string | null;
  per_address_limit: number;
  royalty_ratio: string;
  start_time: Timestamp;
  whitelist_address?: string | null;
}
export interface Coin {
  amount: Uint128;
  denom: string;
  [k: string]: unknown;
}
export type ExecuteMsg = {
  create_minter: {
    msg: InstantiateMsg;
  };
} | {
  update_admin: {
    admin: string;
  };
} | {
  update_fee_collector_address: {
    fee_collector_address: string;
  };
} | {
  update_minter_creation_fee: {
    minter_creation_fee: Coin;
  };
} | {
  update_allowed_minter_mint_denoms: {
    allowed_minter_mint_denoms: string[];
  };
} | {
  update_minter_code_id: {
    minter_code_id: number;
  };
};
export type Timestamp = Uint64;
export type Uint64 = string;
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
export type QueryMsg = {
  params: {};
};
export type Addr = string;
export interface ParamsResponse {
  params: Params;
}
export interface Params {
  admin: Addr;
  allowed_minter_mint_denoms: string[];
  fee_collector_address: Addr;
  minter_code_id: number;
  minter_creation_fee: Coin;
}