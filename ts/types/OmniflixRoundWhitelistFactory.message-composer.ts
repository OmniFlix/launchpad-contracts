/**
* This file was automatically generated by @cosmwasm/ts-codegen@0.35.7.
* DO NOT MODIFY IT BY HAND. Instead, modify the source JSONSchema file,
* and run the @cosmwasm/ts-codegen generate command to regenerate this file.
*/

import { MsgExecuteContractEncodeObject } from "@cosmjs/cosmwasm-stargate";
import { MsgExecuteContract } from "cosmjs-types/cosmwasm/wasm/v1/tx";
import { toUtf8 } from "@cosmjs/encoding";
import { Addr, Uint128, InstantiateMsg, RoundWhitelistFactoryParams, Coin, ExecuteMsg, Timestamp, Uint64, CreateWhitelistMsg, RoundConfig, Round, QueryMsg, Boolean, ParamsResponse, ArrayOfAddr } from "./OmniflixRoundWhitelistFactory.types";
export interface OmniflixRoundWhitelistFactoryMsg {
  contractAddress: string;
  sender: string;
  createWhitelist: ({
    msg
  }: {
    msg: CreateWhitelistMsg;
  }, _funds?: Coin[]) => MsgExecuteContractEncodeObject;
  updateAdmin: ({
    admin
  }: {
    admin: string;
  }, _funds?: Coin[]) => MsgExecuteContractEncodeObject;
  updateFeeCollectorAddress: ({
    feeCollectorAddress
  }: {
    feeCollectorAddress: string;
  }, _funds?: Coin[]) => MsgExecuteContractEncodeObject;
  updateWhitelistCreationFee: ({
    whitelistCreationFee
  }: {
    whitelistCreationFee: Coin;
  }, _funds?: Coin[]) => MsgExecuteContractEncodeObject;
  updateWhitelistCodeId: ({
    whitelistCodeId
  }: {
    whitelistCodeId: number;
  }, _funds?: Coin[]) => MsgExecuteContractEncodeObject;
  pause: (_funds?: Coin[]) => MsgExecuteContractEncodeObject;
  unpause: (_funds?: Coin[]) => MsgExecuteContractEncodeObject;
  setPausers: ({
    pausers
  }: {
    pausers: string[];
  }, _funds?: Coin[]) => MsgExecuteContractEncodeObject;
}
export class OmniflixRoundWhitelistFactoryMsgComposer implements OmniflixRoundWhitelistFactoryMsg {
  sender: string;
  contractAddress: string;

  constructor(sender: string, contractAddress: string) {
    this.sender = sender;
    this.contractAddress = contractAddress;
    this.createWhitelist = this.createWhitelist.bind(this);
    this.updateAdmin = this.updateAdmin.bind(this);
    this.updateFeeCollectorAddress = this.updateFeeCollectorAddress.bind(this);
    this.updateWhitelistCreationFee = this.updateWhitelistCreationFee.bind(this);
    this.updateWhitelistCodeId = this.updateWhitelistCodeId.bind(this);
    this.pause = this.pause.bind(this);
    this.unpause = this.unpause.bind(this);
    this.setPausers = this.setPausers.bind(this);
  }

  createWhitelist = ({
    msg
  }: {
    msg: CreateWhitelistMsg;
  }, _funds?: Coin[]): MsgExecuteContractEncodeObject => {
    return {
      typeUrl: "/cosmwasm.wasm.v1.MsgExecuteContract",
      value: MsgExecuteContract.fromPartial({
        sender: this.sender,
        contract: this.contractAddress,
        msg: toUtf8(JSON.stringify({
          create_whitelist: {
            msg
          }
        })),
        funds: _funds
      })
    };
  };
  updateAdmin = ({
    admin
  }: {
    admin: string;
  }, _funds?: Coin[]): MsgExecuteContractEncodeObject => {
    return {
      typeUrl: "/cosmwasm.wasm.v1.MsgExecuteContract",
      value: MsgExecuteContract.fromPartial({
        sender: this.sender,
        contract: this.contractAddress,
        msg: toUtf8(JSON.stringify({
          update_admin: {
            admin
          }
        })),
        funds: _funds
      })
    };
  };
  updateFeeCollectorAddress = ({
    feeCollectorAddress
  }: {
    feeCollectorAddress: string;
  }, _funds?: Coin[]): MsgExecuteContractEncodeObject => {
    return {
      typeUrl: "/cosmwasm.wasm.v1.MsgExecuteContract",
      value: MsgExecuteContract.fromPartial({
        sender: this.sender,
        contract: this.contractAddress,
        msg: toUtf8(JSON.stringify({
          update_fee_collector_address: {
            fee_collector_address: feeCollectorAddress
          }
        })),
        funds: _funds
      })
    };
  };
  updateWhitelistCreationFee = ({
    whitelistCreationFee
  }: {
    whitelistCreationFee: Coin;
  }, _funds?: Coin[]): MsgExecuteContractEncodeObject => {
    return {
      typeUrl: "/cosmwasm.wasm.v1.MsgExecuteContract",
      value: MsgExecuteContract.fromPartial({
        sender: this.sender,
        contract: this.contractAddress,
        msg: toUtf8(JSON.stringify({
          update_whitelist_creation_fee: {
            whitelist_creation_fee: whitelistCreationFee
          }
        })),
        funds: _funds
      })
    };
  };
  updateWhitelistCodeId = ({
    whitelistCodeId
  }: {
    whitelistCodeId: number;
  }, _funds?: Coin[]): MsgExecuteContractEncodeObject => {
    return {
      typeUrl: "/cosmwasm.wasm.v1.MsgExecuteContract",
      value: MsgExecuteContract.fromPartial({
        sender: this.sender,
        contract: this.contractAddress,
        msg: toUtf8(JSON.stringify({
          update_whitelist_code_id: {
            whitelist_code_id: whitelistCodeId
          }
        })),
        funds: _funds
      })
    };
  };
  pause = (_funds?: Coin[]): MsgExecuteContractEncodeObject => {
    return {
      typeUrl: "/cosmwasm.wasm.v1.MsgExecuteContract",
      value: MsgExecuteContract.fromPartial({
        sender: this.sender,
        contract: this.contractAddress,
        msg: toUtf8(JSON.stringify({
          pause: {}
        })),
        funds: _funds
      })
    };
  };
  unpause = (_funds?: Coin[]): MsgExecuteContractEncodeObject => {
    return {
      typeUrl: "/cosmwasm.wasm.v1.MsgExecuteContract",
      value: MsgExecuteContract.fromPartial({
        sender: this.sender,
        contract: this.contractAddress,
        msg: toUtf8(JSON.stringify({
          unpause: {}
        })),
        funds: _funds
      })
    };
  };
  setPausers = ({
    pausers
  }: {
    pausers: string[];
  }, _funds?: Coin[]): MsgExecuteContractEncodeObject => {
    return {
      typeUrl: "/cosmwasm.wasm.v1.MsgExecuteContract",
      value: MsgExecuteContract.fromPartial({
        sender: this.sender,
        contract: this.contractAddress,
        msg: toUtf8(JSON.stringify({
          set_pausers: {
            pausers
          }
        })),
        funds: _funds
      })
    };
  };
}