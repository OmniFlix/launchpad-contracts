/**
* This file was automatically generated by @cosmwasm/ts-codegen@0.35.3.
* DO NOT MODIFY IT BY HAND. Instead, modify the source JSONSchema file,
* and run the @cosmwasm/ts-codegen generate command to regenerate this file.
*/

export type Uint128 = string;
export interface InstantiateMsg {
  admin?: string | null;
  rounds: Round[];
}
export interface Coin {
  amount: Uint128;
  denom: string;
  [k: string]: unknown;
}
export type ExecuteMsg = {
  create_whitelist: {
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
  update_whitelist_creation_fee: {
    whitelist_creation_fee: Coin;
  };
} | {
  update_whitelist_code_id: {
    whitelist_code_id: number;
  };
};
export type Addr = string;
export type Timestamp = Uint64;
export type Uint64 = string;
export interface Round {
  addresses: Addr[];
  end_time: Timestamp;
  mint_price: Coin;
  round_per_address_limit: number;
  start_time: Timestamp;
}
export type QueryMsg = {
  params: {};
};
export interface ParamsResponse {
  params: Params;
}
export interface Params {
  admin: Addr;
  fee_collector_address: Addr;
  whitelist_code_id: number;
  whitelist_creation_fee: Coin;
}