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
  init: OpenEditionMinterInitExtention;
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
export interface OpenEditionMinterInitExtention {
  admin: string;
  end_time?: Timestamp | null;
  mint_price: Coin;
  payment_collector?: string | null;
  per_address_limit: number;
  royalty_ratio: string;
  start_time: Timestamp;
  token_limit?: number | null;
  whitelist_address?: string | null;
}
export interface Coin {
  amount: Uint128;
  denom: string;
  [k: string]: unknown;
}
export type ExecuteMsg = {
  mint: {
    drop_id?: number | null;
  };
} | {
  mint_admin: {
    drop_id?: number | null;
    recipient: string;
  };
} | {
  update_royalty_ratio: {
    drop_id?: number | null;
    ratio: string;
  };
} | {
  update_mint_price: {
    drop_id?: number | null;
    mint_price: Coin;
  };
} | {
  update_whitelist_address: {
    address: string;
    drop_id?: number | null;
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
  new_drop: {
    base_uri?: string | null;
    data?: string | null;
    description?: string | null;
    end_time?: Timestamp | null;
    extensible?: boolean | null;
    mint_price: Coin;
    nsfw?: boolean | null;
    per_address_limit: number;
    preview_uri?: string | null;
    royalty_ratio?: string | null;
    start_time: Timestamp;
    token_limit?: number | null;
    token_name: string;
    transferable?: boolean | null;
    uri_hash?: string | null;
    whitelist_address?: string | null;
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
  collection: {
    drop_id?: number | null;
  };
} | {
  config: {
    drop_id?: number | null;
  };
} | {
  minted_tokens: {
    address: string;
    drop_id?: number | null;
  };
} | {
  total_minted_count: {
    drop_id?: number | null;
  };
} | {
  tokens_remaining: {
    drop_id?: number | null;
  };
} | {
  is_paused: {};
} | {
  pausers: {};
} | {
  current_drop_number: {};
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
export type Uint32 = number;
export type Boolean = boolean;
export interface UserDetails {
  minted_tokens: Token[];
  public_mint_count: number;
  total_minted_count: number;
}
export interface Token {
  token_id: string;
}
export type ArrayOfAddr = Addr[];