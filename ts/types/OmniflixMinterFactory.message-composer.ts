/**
* This file was automatically generated by @cosmwasm/ts-codegen@0.35.7.
* DO NOT MODIFY IT BY HAND. Instead, modify the source JSONSchema file,
* and run the @cosmwasm/ts-codegen generate command to regenerate this file.
*/

import { MsgExecuteContractEncodeObject } from "@cosmjs/cosmwasm-stargate";
import { MsgExecuteContract } from "cosmjs-types/cosmwasm/wasm/v1/tx";
import { toUtf8 } from "@cosmjs/encoding";
import { Addr, Uint128, InstantiateMsg, MinterFactoryParams, Coin, ExecuteMsg, Timestamp, Uint64, Decimal, MinterInstantiateMsgForMinterInitExtention, CollectionDetails, WeightedAddress, MinterInitExtention, TokenDetails, QueryMsg, ParamsResponse } from "./OmniflixMinterFactory.types";
export interface OmniflixMinterFactoryMsg {
  contractAddress: string;
  sender: string;
  createMinter: ({
    msg
  }: {
    msg: MinterInstantiateMsgForMinterInitExtention;
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
  updateMinterCreationFee: ({
    minterCreationFee
  }: {
    minterCreationFee: Coin;
  }, _funds?: Coin[]) => MsgExecuteContractEncodeObject;
  updateMinterCodeId: ({
    minterCodeId
  }: {
    minterCodeId: number;
  }, _funds?: Coin[]) => MsgExecuteContractEncodeObject;
}
export class OmniflixMinterFactoryMsgComposer implements OmniflixMinterFactoryMsg {
  sender: string;
  contractAddress: string;

  constructor(sender: string, contractAddress: string) {
    this.sender = sender;
    this.contractAddress = contractAddress;
    this.createMinter = this.createMinter.bind(this);
    this.updateAdmin = this.updateAdmin.bind(this);
    this.updateFeeCollectorAddress = this.updateFeeCollectorAddress.bind(this);
    this.updateMinterCreationFee = this.updateMinterCreationFee.bind(this);
    this.updateMinterCodeId = this.updateMinterCodeId.bind(this);
  }

  createMinter = ({
    msg
  }: {
    msg: MinterInstantiateMsgForMinterInitExtention;
  }, _funds?: Coin[]): MsgExecuteContractEncodeObject => {
    return {
      typeUrl: "/cosmwasm.wasm.v1.MsgExecuteContract",
      value: MsgExecuteContract.fromPartial({
        sender: this.sender,
        contract: this.contractAddress,
        msg: toUtf8(JSON.stringify({
          create_minter: {
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
  updateMinterCreationFee = ({
    minterCreationFee
  }: {
    minterCreationFee: Coin;
  }, _funds?: Coin[]): MsgExecuteContractEncodeObject => {
    return {
      typeUrl: "/cosmwasm.wasm.v1.MsgExecuteContract",
      value: MsgExecuteContract.fromPartial({
        sender: this.sender,
        contract: this.contractAddress,
        msg: toUtf8(JSON.stringify({
          update_minter_creation_fee: {
            minter_creation_fee: minterCreationFee
          }
        })),
        funds: _funds
      })
    };
  };
  updateMinterCodeId = ({
    minterCodeId
  }: {
    minterCodeId: number;
  }, _funds?: Coin[]): MsgExecuteContractEncodeObject => {
    return {
      typeUrl: "/cosmwasm.wasm.v1.MsgExecuteContract",
      value: MsgExecuteContract.fromPartial({
        sender: this.sender,
        contract: this.contractAddress,
        msg: toUtf8(JSON.stringify({
          update_minter_code_id: {
            minter_code_id: minterCodeId
          }
        })),
        funds: _funds
      })
    };
  };
}