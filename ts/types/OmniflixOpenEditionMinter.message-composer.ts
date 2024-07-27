/**
* This file was automatically generated by @cosmwasm/ts-codegen@0.35.7.
* DO NOT MODIFY IT BY HAND. Instead, modify the source JSONSchema file,
* and run the @cosmwasm/ts-codegen generate command to regenerate this file.
*/

import { MsgExecuteContractEncodeObject } from "@cosmjs/cosmwasm-stargate";
import { MsgExecuteContract } from "cosmjs-types/cosmwasm/wasm/v1/tx";
import { toUtf8 } from "@cosmjs/encoding";
import { Addr, Timestamp, Uint64, Uint128, Decimal, InstantiateMsg, AuthDetails, CollectionDetails, WeightedAddress, OpenEditionMinterInitExtention, Coin, TokenDetails, ExecuteMsg, QueryMsg, OEMQueryExtension, Config, Uint32, Boolean, MintHistoryResponse, ArrayOfAddr, UserDetails, Token } from "./OmniflixOpenEditionMinter.types";
export interface OmniflixOpenEditionMinterMsg {
  contractAddress: string;
  sender: string;
  mint: (_funds?: Coin[]) => MsgExecuteContractEncodeObject;
  mintAdmin: ({
    recipient
  }: {
    recipient: string;
  }, _funds?: Coin[]) => MsgExecuteContractEncodeObject;
  updateRoyaltyRatio: ({
    ratio
  }: {
    ratio: string;
  }, _funds?: Coin[]) => MsgExecuteContractEncodeObject;
  updateMintPrice: ({
    mintPrice
  }: {
    mintPrice: Coin;
  }, _funds?: Coin[]) => MsgExecuteContractEncodeObject;
  updateWhitelistAddress: ({
    address
  }: {
    address: string;
  }, _funds?: Coin[]) => MsgExecuteContractEncodeObject;
  pause: (_funds?: Coin[]) => MsgExecuteContractEncodeObject;
  unpause: (_funds?: Coin[]) => MsgExecuteContractEncodeObject;
  setPausers: ({
    pausers
  }: {
    pausers: string[];
  }, _funds?: Coin[]) => MsgExecuteContractEncodeObject;
  updateRoyaltyReceivers: ({
    receivers
  }: {
    receivers: WeightedAddress[];
  }, _funds?: Coin[]) => MsgExecuteContractEncodeObject;
  updateDenom: ({
    collectionName,
    description,
    previewUri
  }: {
    collectionName?: string;
    description?: string;
    previewUri?: string;
  }, _funds?: Coin[]) => MsgExecuteContractEncodeObject;
  purgeDenom: (_funds?: Coin[]) => MsgExecuteContractEncodeObject;
  updateAdmin: ({
    admin
  }: {
    admin: string;
  }, _funds?: Coin[]) => MsgExecuteContractEncodeObject;
  updatePaymentCollector: ({
    paymentCollector
  }: {
    paymentCollector: string;
  }, _funds?: Coin[]) => MsgExecuteContractEncodeObject;
  burnRemainingTokens: (_funds?: Coin[]) => MsgExecuteContractEncodeObject;
}
export class OmniflixOpenEditionMinterMsgComposer implements OmniflixOpenEditionMinterMsg {
  sender: string;
  contractAddress: string;

  constructor(sender: string, contractAddress: string) {
    this.sender = sender;
    this.contractAddress = contractAddress;
    this.mint = this.mint.bind(this);
    this.mintAdmin = this.mintAdmin.bind(this);
    this.updateRoyaltyRatio = this.updateRoyaltyRatio.bind(this);
    this.updateMintPrice = this.updateMintPrice.bind(this);
    this.updateWhitelistAddress = this.updateWhitelistAddress.bind(this);
    this.pause = this.pause.bind(this);
    this.unpause = this.unpause.bind(this);
    this.setPausers = this.setPausers.bind(this);
    this.updateRoyaltyReceivers = this.updateRoyaltyReceivers.bind(this);
    this.updateDenom = this.updateDenom.bind(this);
    this.purgeDenom = this.purgeDenom.bind(this);
    this.updateAdmin = this.updateAdmin.bind(this);
    this.updatePaymentCollector = this.updatePaymentCollector.bind(this);
    this.burnRemainingTokens = this.burnRemainingTokens.bind(this);
  }

  mint = (_funds?: Coin[]): MsgExecuteContractEncodeObject => {
    return {
      typeUrl: "/cosmwasm.wasm.v1.MsgExecuteContract",
      value: MsgExecuteContract.fromPartial({
        sender: this.sender,
        contract: this.contractAddress,
        msg: toUtf8(JSON.stringify({
          mint: {}
        })),
        funds: _funds
      })
    };
  };
  mintAdmin = ({
    recipient
  }: {
    recipient: string;
  }, _funds?: Coin[]): MsgExecuteContractEncodeObject => {
    return {
      typeUrl: "/cosmwasm.wasm.v1.MsgExecuteContract",
      value: MsgExecuteContract.fromPartial({
        sender: this.sender,
        contract: this.contractAddress,
        msg: toUtf8(JSON.stringify({
          mint_admin: {
            recipient
          }
        })),
        funds: _funds
      })
    };
  };
  updateRoyaltyRatio = ({
    ratio
  }: {
    ratio: string;
  }, _funds?: Coin[]): MsgExecuteContractEncodeObject => {
    return {
      typeUrl: "/cosmwasm.wasm.v1.MsgExecuteContract",
      value: MsgExecuteContract.fromPartial({
        sender: this.sender,
        contract: this.contractAddress,
        msg: toUtf8(JSON.stringify({
          update_royalty_ratio: {
            ratio
          }
        })),
        funds: _funds
      })
    };
  };
  updateMintPrice = ({
    mintPrice
  }: {
    mintPrice: Coin;
  }, _funds?: Coin[]): MsgExecuteContractEncodeObject => {
    return {
      typeUrl: "/cosmwasm.wasm.v1.MsgExecuteContract",
      value: MsgExecuteContract.fromPartial({
        sender: this.sender,
        contract: this.contractAddress,
        msg: toUtf8(JSON.stringify({
          update_mint_price: {
            mint_price: mintPrice
          }
        })),
        funds: _funds
      })
    };
  };
  updateWhitelistAddress = ({
    address
  }: {
    address: string;
  }, _funds?: Coin[]): MsgExecuteContractEncodeObject => {
    return {
      typeUrl: "/cosmwasm.wasm.v1.MsgExecuteContract",
      value: MsgExecuteContract.fromPartial({
        sender: this.sender,
        contract: this.contractAddress,
        msg: toUtf8(JSON.stringify({
          update_whitelist_address: {
            address
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
  updateRoyaltyReceivers = ({
    receivers
  }: {
    receivers: WeightedAddress[];
  }, _funds?: Coin[]): MsgExecuteContractEncodeObject => {
    return {
      typeUrl: "/cosmwasm.wasm.v1.MsgExecuteContract",
      value: MsgExecuteContract.fromPartial({
        sender: this.sender,
        contract: this.contractAddress,
        msg: toUtf8(JSON.stringify({
          update_royalty_receivers: {
            receivers
          }
        })),
        funds: _funds
      })
    };
  };
  updateDenom = ({
    collectionName,
    description,
    previewUri
  }: {
    collectionName?: string;
    description?: string;
    previewUri?: string;
  }, _funds?: Coin[]): MsgExecuteContractEncodeObject => {
    return {
      typeUrl: "/cosmwasm.wasm.v1.MsgExecuteContract",
      value: MsgExecuteContract.fromPartial({
        sender: this.sender,
        contract: this.contractAddress,
        msg: toUtf8(JSON.stringify({
          update_denom: {
            collection_name: collectionName,
            description,
            preview_uri: previewUri
          }
        })),
        funds: _funds
      })
    };
  };
  purgeDenom = (_funds?: Coin[]): MsgExecuteContractEncodeObject => {
    return {
      typeUrl: "/cosmwasm.wasm.v1.MsgExecuteContract",
      value: MsgExecuteContract.fromPartial({
        sender: this.sender,
        contract: this.contractAddress,
        msg: toUtf8(JSON.stringify({
          purge_denom: {}
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
  updatePaymentCollector = ({
    paymentCollector
  }: {
    paymentCollector: string;
  }, _funds?: Coin[]): MsgExecuteContractEncodeObject => {
    return {
      typeUrl: "/cosmwasm.wasm.v1.MsgExecuteContract",
      value: MsgExecuteContract.fromPartial({
        sender: this.sender,
        contract: this.contractAddress,
        msg: toUtf8(JSON.stringify({
          update_payment_collector: {
            payment_collector: paymentCollector
          }
        })),
        funds: _funds
      })
    };
  };
  burnRemainingTokens = (_funds?: Coin[]): MsgExecuteContractEncodeObject => {
    return {
      typeUrl: "/cosmwasm.wasm.v1.MsgExecuteContract",
      value: MsgExecuteContract.fromPartial({
        sender: this.sender,
        contract: this.contractAddress,
        msg: toUtf8(JSON.stringify({
          burn_remaining_tokens: {}
        })),
        funds: _funds
      })
    };
  };
}