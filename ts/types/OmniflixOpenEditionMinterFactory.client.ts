/**
* This file was automatically generated by @cosmwasm/ts-codegen@0.35.7.
* DO NOT MODIFY IT BY HAND. Instead, modify the source JSONSchema file,
* and run the @cosmwasm/ts-codegen generate command to regenerate this file.
*/

import { CosmWasmClient, SigningCosmWasmClient, ExecuteResult } from "@cosmjs/cosmwasm-stargate";
import { StdFee } from "@cosmjs/amino";
import { Uint128, InstantiateMsg, Coin, ExecuteMsg, Timestamp, Uint64, MinterInstantiateMsgForOpenEditionMinterInitExtention, CollectionDetails, OpenEditionMinterInitExtention, QueryMsg, Addr, ParamsResponse, Params } from "./OmniflixOpenEditionMinterFactory.types";
export interface OmniflixOpenEditionMinterFactoryReadOnlyInterface {
  contractAddress: string;
  params: () => Promise<ParamsResponse>;
}
export class OmniflixOpenEditionMinterFactoryQueryClient implements OmniflixOpenEditionMinterFactoryReadOnlyInterface {
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
export interface OmniflixOpenEditionMinterFactoryInterface extends OmniflixOpenEditionMinterFactoryReadOnlyInterface {
  contractAddress: string;
  sender: string;
  createMinter: ({
    msg
  }: {
    msg: MinterInstantiateMsgForOpenEditionMinterInitExtention;
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
  updateMinterCreationFee: ({
    minterCreationFee
  }: {
    minterCreationFee: Coin;
  }, fee?: number | StdFee | "auto", memo?: string, _funds?: Coin[]) => Promise<ExecuteResult>;
  updateMinterCodeId: ({
    minterCodeId
  }: {
    minterCodeId: number;
  }, fee?: number | StdFee | "auto", memo?: string, _funds?: Coin[]) => Promise<ExecuteResult>;
}
export class OmniflixOpenEditionMinterFactoryClient extends OmniflixOpenEditionMinterFactoryQueryClient implements OmniflixOpenEditionMinterFactoryInterface {
  client: SigningCosmWasmClient;
  sender: string;
  contractAddress: string;

  constructor(client: SigningCosmWasmClient, sender: string, contractAddress: string) {
    super(client, contractAddress);
    this.client = client;
    this.sender = sender;
    this.contractAddress = contractAddress;
    this.createMinter = this.createMinter.bind(this);
    this.updateAdmin = this.updateAdmin.bind(this);
    this.updateFeeCollectorAddress = this.updateFeeCollectorAddress.bind(this);
    this.updateMinterCreationFee = this.updateMinterCreationFee.bind(this);
    this.updateMinterCodeId = this.updateMinterCodeId.bind(this);
  }

  createMinter = async ({
    msg
  }: {
    msg: MinterInstantiateMsgForOpenEditionMinterInitExtention;
  }, fee: number | StdFee | "auto" = "auto", memo?: string, _funds?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      create_minter: {
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
  updateMinterCreationFee = async ({
    minterCreationFee
  }: {
    minterCreationFee: Coin;
  }, fee: number | StdFee | "auto" = "auto", memo?: string, _funds?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      update_minter_creation_fee: {
        minter_creation_fee: minterCreationFee
      }
    }, fee, memo, _funds);
  };
  updateMinterCodeId = async ({
    minterCodeId
  }: {
    minterCodeId: number;
  }, fee: number | StdFee | "auto" = "auto", memo?: string, _funds?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      update_minter_code_id: {
        minter_code_id: minterCodeId
      }
    }, fee, memo, _funds);
  };
}