/**
* This file was automatically generated by @cosmwasm/ts-codegen@0.35.7.
* DO NOT MODIFY IT BY HAND. Instead, modify the source JSONSchema file,
* and run the @cosmwasm/ts-codegen generate command to regenerate this file.
*/

export type Uint128 = string;
export interface InstantiateMsg {
  admin?: string | null;
  fee_collector_address: string;
  minter_creation_fee: Coin;
  open_edition_minter_code_id: number;
}
export interface Coin {
  amount: Uint128;
  denom: string;
  [k: string]: unknown;
}
export type ExecuteMsg = {
  create_minter: {
    msg: MinterInstantiateMsgForOpenEditionMinterInitExtention;
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
  update_minter_code_id: {
    minter_code_id: number;
  };
};
export type Timestamp = Uint64;
export type Uint64 = string;
export type Decimal = string;
export interface MinterInstantiateMsgForOpenEditionMinterInitExtention {
  collection_details: CollectionDetails;
  init: OpenEditionMinterInitExtention;
  token_details: TokenDetails;
}
export interface CollectionDetails {
  collection_name: string;
  data?: string | null;
  description?: string | null;
  id: string;
  preview_uri?: string | null;
  royalty_receivers?: WeightedAddress[] | null;
  schema?: string | null;
  symbol: string;
  uri?: string | null;
  uri_hash?: string | null;
}
export interface WeightedAddress {
  address: string;
  weight: string;
  [k: string]: unknown;
}
export interface OpenEditionMinterInitExtention {
  admin: string;
  end_time?: Timestamp | null;
  mint_price: Coin;
  num_tokens?: number | null;
  payment_collector?: string | null;
  per_address_limit: number;
  start_time: Timestamp;
  whitelist_address?: string | null;
}
export interface TokenDetails {
  base_token_uri: string;
  data?: string | null;
  description?: string | null;
  extensible: boolean;
  nsfw: boolean;
  preview_uri?: string | null;
  royalty_ratio: Decimal;
  token_name: string;
  transferable: boolean;
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
  fee_collector_address: Addr;
  minter_creation_fee: Coin;
  open_edition_minter_code_id: number;
}