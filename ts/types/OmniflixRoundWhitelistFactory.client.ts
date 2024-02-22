/**
* This file was automatically generated by @cosmwasm/ts-codegen@0.35.7.
* DO NOT MODIFY IT BY HAND. Instead, modify the source JSONSchema file,
* and run the @cosmwasm/ts-codegen generate command to regenerate this file.
*/

import { CosmWasmClient, SigningCosmWasmClient, ExecuteResult } from "@cosmjs/cosmwasm-stargate";
import { StdFee } from "@cosmjs/amino";
import { Addr, Uint128, InstantiateMsg, FactoryParamsForEmpty, Coin, Empty, ExecuteMsg, Timestamp, Uint64, Round, QueryMsg, ParamsResponse } from "./OmniflixRoundWhitelistFactory.types";
export interface OmniflixRoundWhitelistFactoryReadOnlyInterface {
  contractAddress: string;
  params: () => Promise<ParamsResponse>;
}
export class OmniflixRoundWhitelistFactoryQueryClient implements OmniflixRoundWhitelistFactoryReadOnlyInterface {
  client: CosmWasmClient;
  contractAddress: string;

  constructor(client: CosmWasmClient, contractAddress: string) {
    this.client = client;
    this.contractAddress = contractAddress;
    this.params = this.params.bind(this);
  }

  params = async (): Promise<ParamsResponse> => {
    return this.client.queryContractSmart(this.contractAddress, {
      params: {}
    });
  };
}
export interface OmniflixRoundWhitelistFactoryInterface extends OmniflixRoundWhitelistFactoryReadOnlyInterface {
  contractAddress: string;
  sender: string;
  createWhitelist: ({
    msg
  }: {
    msg: InstantiateMsg;
  }, fee?: number | StdFee | "auto", memo?: string, _funds?: Coin[]) => Promise<ExecuteResult>;
  updateAdmin: ({
    admin
  }: {
    admin: string;
  }, fee?: number | StdFee | "auto", memo?: string, _funds?: Coin[]) => Promise<ExecuteResult>;
  updateFeeCollectorAddress: ({
    feeCollectorAddress
  }: {
    feeCollectorAddress: string;
  }, fee?: number | StdFee | "auto", memo?: string, _funds?: Coin[]) => Promise<ExecuteResult>;
  updateWhitelistCreationFee: ({
    whitelistCreationFee
  }: {
    whitelistCreationFee: Coin;
  }, fee?: number | StdFee | "auto", memo?: string, _funds?: Coin[]) => Promise<ExecuteResult>;
  updateWhitelistCodeId: ({
    whitelistCodeId
  }: {
    whitelistCodeId: number;
  }, fee?: number | StdFee | "auto", memo?: string, _funds?: Coin[]) => Promise<ExecuteResult>;
}
export class OmniflixRoundWhitelistFactoryClient extends OmniflixRoundWhitelistFactoryQueryClient implements OmniflixRoundWhitelistFactoryInterface {
  client: SigningCosmWasmClient;
  sender: string;
  contractAddress: string;

  constructor(client: SigningCosmWasmClient, sender: string, contractAddress: string) {
    super(client, contractAddress);
    this.client = client;
    this.sender = sender;
    this.contractAddress = contractAddress;
    this.createWhitelist = this.createWhitelist.bind(this);
    this.updateAdmin = this.updateAdmin.bind(this);
    this.updateFeeCollectorAddress = this.updateFeeCollectorAddress.bind(this);
    this.updateWhitelistCreationFee = this.updateWhitelistCreationFee.bind(this);
    this.updateWhitelistCodeId = this.updateWhitelistCodeId.bind(this);
  }

  createWhitelist = async ({
    msg
  }: {
    msg: InstantiateMsg;
  }, fee: number | StdFee | "auto" = "auto", memo?: string, _funds?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      create_whitelist: {
        msg
      }
    }, fee, memo, _funds);
  };
  updateAdmin = async ({
    admin
  }: {
    admin: string;
  }, fee: number | StdFee | "auto" = "auto", memo?: string, _funds?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      update_admin: {
        admin
      }
    }, fee, memo, _funds);
  };
  updateFeeCollectorAddress = async ({
    feeCollectorAddress
  }: {
    feeCollectorAddress: string;
  }, fee: number | StdFee | "auto" = "auto", memo?: string, _funds?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      update_fee_collector_address: {
        fee_collector_address: feeCollectorAddress
      }
    }, fee, memo, _funds);
  };
  updateWhitelistCreationFee = async ({
    whitelistCreationFee
  }: {
    whitelistCreationFee: Coin;
  }, fee: number | StdFee | "auto" = "auto", memo?: string, _funds?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      update_whitelist_creation_fee: {
        whitelist_creation_fee: whitelistCreationFee
      }
    }, fee, memo, _funds);
  };
  updateWhitelistCodeId = async ({
    whitelistCodeId
  }: {
    whitelistCodeId: number;
  }, fee: number | StdFee | "auto" = "auto", memo?: string, _funds?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      update_whitelist_code_id: {
        whitelist_code_id: whitelistCodeId
      }
    }, fee, memo, _funds);
  };
}