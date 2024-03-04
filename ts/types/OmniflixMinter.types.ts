/**
* This file was automatically generated by @cosmwasm/ts-codegen@0.35.7.
* DO NOT MODIFY IT BY HAND. Instead, modify the source JSONSchema file,
* and run the @cosmwasm/ts-codegen generate command to regenerate this file.
*/

export type Timestamp = Uint64;
export type Uint64 = string;
export type Uint128 = string;
export type Decimal = string;
export interface InstantiateMsg {
  collection_details: CollectionDetails;
  init: MinterInitExtention;
  token_details?: TokenDetails | null;
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
export interface MinterInitExtention {
  admin: string;
  end_time?: Timestamp | null;
  mint_price: Coin;
  num_tokens: number;
  payment_collector?: string | null;
  per_address_limit?: number | null;
  start_time: Timestamp;
  whitelist_address?: string | null;
}
export interface Coin {
  amount: Uint128;
  denom: string;
  [k: string]: unknown;
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
    collection_name?: string | null;
    description?: string | null;
    preview_uri?: string | null;
  };
} | {
  purge_denom: {};
} | {
  set_admin: {
    admin: string;
  };
} | {
  set_payment_collector: {
    payment_collector: string;
  };
};
export type QueryMsg = {
  collection: {};
} | {
  token_details: {};
} | {
  auth_details: {};
} | {
  config: {};
} | {
  user_minting_details: {
    address: string;
  };
} | {
  is_paused: {};
} | {
  pausers: {};
} | {
  extension: MinterExtensionQueryMsg;
} | {
  total_minted_count: {};
};
export type MinterExtensionQueryMsg = {
  mintable_tokens: {};
} | {
  total_tokens_remaining: {};
};
export type Addr = string;
export interface AuthDetails {
  admin: Addr;
  payment_collector: Addr;
}
export interface Config {
  end_time?: Timestamp | null;
  mint_price: Coin;
  num_tokens?: number | null;
  per_address_limit?: number | null;
  start_time: Timestamp;
  whitelist_address?: Addr | null;
}
export type Uint32 = number;
export type Boolean = boolean;
export type ArrayOfAddr = Addr[];
export interface UserDetails {
  minted_tokens: Token[];
  public_mint_count: number;
  total_minted_count: number;
}
export interface Token {
  token_id: string;
}