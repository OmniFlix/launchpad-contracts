/**
* This file was automatically generated by @cosmwasm/ts-codegen@0.35.7.
* DO NOT MODIFY IT BY HAND. Instead, modify the source JSONSchema file,
* and run the @cosmwasm/ts-codegen generate command to regenerate this file.
*/

export type Timestamp = Uint64;
export type Uint64 = string;
export type Uint128 = string;
export interface InstantiateMsg {
  collection_details: CollectionDetails;
  init: MinterInitExtention;
}
export interface CollectionDetails {
  base_uri: string;
  data: string;
  description: string;
  extensible: boolean;
  id: string;
  name: string;
  nsfw: boolean;
  preview_uri: string;
  royalty_receivers?: WeightedAddress[] | null;
  schema: string;
  symbol: string;
  token_name: string;
  transferable: boolean;
  uri: string;
  uri_hash: string;
}
export interface WeightedAddress {
  address: string;
  weight: string;
  [k: string]: unknown;
}
export interface MinterInitExtention {
  admin: string;
  end_time?: Timestamp | null;
  mint_price: Coin;
  num_tokens: number;
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
  mint: {};
} | {
  mint_admin: {
    recipient: string;
    token_id?: string | null;
  };
} | {
  burn_remaining_tokens: {};
} | {
  update_royalty_ratio: {
    ratio: string;
  };
} | {
  update_mint_price: {
    mint_price: Coin;
  };
} | {
  randomize_list: {};
} | {
  update_whitelist_address: {
    address: string;
  };
} | {
  pause: {};
} | {
  unpause: {};
} | {
  set_pausers: {
    pausers: string[];
  };
} | {
  update_royalty_receivers: {
    receivers: WeightedAddress[];
  };
} | {
  update_denom: {
    description?: string | null;
    name?: string | null;
    preview_uri?: string | null;
  };
} | {
  purge_denom: {};
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
} | {
  is_paused: {};
} | {
  pausers: {};
};
export type Addr = string;
export type Decimal = string;
export interface Config {
  admin: Addr;
  end_time?: Timestamp | null;
  mint_price: Coin;
  payment_collector: Addr;
  per_address_limit: number;
  royalty_ratio: Decimal;
  start_time: Timestamp;
  token_limit?: number | null;
  whitelist_address?: Addr | null;
}
export type Boolean = boolean;
export type ArrayOfToken = Token[];
export interface Token {
  token_id: string;
}
export interface UserDetails {
  minted_tokens: Token[];
  public_mint_count: number;
  total_minted_count: number;
}
export type ArrayOfAddr = Addr[];
export type Uint32 = number;