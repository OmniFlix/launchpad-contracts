/**
* This file was automatically generated by @cosmwasm/ts-codegen@0.35.7.
* DO NOT MODIFY IT BY HAND. Instead, modify the source JSONSchema file,
* and run the @cosmwasm/ts-codegen generate command to regenerate this file.
*/

import { CosmWasmClient, SigningCosmWasmClient, ExecuteResult } from "@cosmjs/cosmwasm-stargate";
import { StdFee } from "@cosmjs/amino";
import { Timestamp, Uint64, Uint128, InstantiateMsg, CollectionDetails, WeightedAddress, OpenEditionMinterInitExtention, Coin, ExecuteMsg, QueryMsg, Addr, Decimal, Config, Boolean, UserDetails, Token, ArrayOfAddr, Uint32 } from "./OmniflixOpenEditionMinter.types";
export interface OmniflixOpenEditionMinterReadOnlyInterface {
  contractAddress: string;
  collection: () => Promise<CollectionDetails>;
  config: () => Promise<Config>;
  mintedTokens: ({
    address
  }: {
    address: string;
  }) => Promise<UserDetails>;
  totalMintedCount: () => Promise<Uint32>;
  tokensRemaining: () => Promise<Uint32>;
  isPaused: () => Promise<Boolean>;
  pausers: () => Promise<ArrayOfAddr>;
}
export class OmniflixOpenEditionMinterQueryClient implements OmniflixOpenEditionMinterReadOnlyInterface {
  client: CosmWasmClient;
  contractAddress: string;

  constructor(client: CosmWasmClient, contractAddress: string) {
    this.client = client;
    this.contractAddress = contractAddress;
    this.collection = this.collection.bind(this);
    this.config = this.config.bind(this);
    this.mintedTokens = this.mintedTokens.bind(this);
    this.totalMintedCount = this.totalMintedCount.bind(this);
    this.tokensRemaining = this.tokensRemaining.bind(this);
    this.isPaused = this.isPaused.bind(this);
    this.pausers = this.pausers.bind(this);
  }

  collection = async (): Promise<CollectionDetails> => {
    return this.client.queryContractSmart(this.contractAddress, {
      collection: {}
    });
  };
  config = async (): Promise<Config> => {
    return this.client.queryContractSmart(this.contractAddress, {
      config: {}
    });
  };
  mintedTokens = async ({
    address
  }: {
    address: string;
  }): Promise<UserDetails> => {
    return this.client.queryContractSmart(this.contractAddress, {
      minted_tokens: {
        address
      }
    });
  };
  totalMintedCount = async (): Promise<Uint32> => {
    return this.client.queryContractSmart(this.contractAddress, {
      total_minted_count: {}
    });
  };
  tokensRemaining = async (): Promise<Uint32> => {
    return this.client.queryContractSmart(this.contractAddress, {
      tokens_remaining: {}
    });
  };
  isPaused = async (): Promise<Boolean> => {
    return this.client.queryContractSmart(this.contractAddress, {
      is_paused: {}
    });
  };
  pausers = async (): Promise<ArrayOfAddr> => {
    return this.client.queryContractSmart(this.contractAddress, {
      pausers: {}
    });
  };
}
export interface OmniflixOpenEditionMinterInterface extends OmniflixOpenEditionMinterReadOnlyInterface {
  contractAddress: string;
  sender: string;
  mint: (fee?: number | StdFee | "auto", memo?: string, _funds?: Coin[]) => Promise<ExecuteResult>;
  mintAdmin: ({
    recipient
  }: {
    recipient: string;
  }, fee?: number | StdFee | "auto", memo?: string, _funds?: Coin[]) => Promise<ExecuteResult>;
  updateRoyaltyRatio: ({
    ratio
  }: {
    ratio: string;
  }, fee?: number | StdFee | "auto", memo?: string, _funds?: Coin[]) => Promise<ExecuteResult>;
  updateMintPrice: ({
    mintPrice
  }: {
    mintPrice: Coin;
  }, fee?: number | StdFee | "auto", memo?: string, _funds?: Coin[]) => Promise<ExecuteResult>;
  updateWhitelistAddress: ({
    address
  }: {
    address: string;
  }, fee?: number | StdFee | "auto", memo?: string, _funds?: Coin[]) => Promise<ExecuteResult>;
  pause: (fee?: number | StdFee | "auto", memo?: string, _funds?: Coin[]) => Promise<ExecuteResult>;
  unpause: (fee?: number | StdFee | "auto", memo?: string, _funds?: Coin[]) => Promise<ExecuteResult>;
  setPausers: ({
    pausers
  }: {
    pausers: string[];
  }, fee?: number | StdFee | "auto", memo?: string, _funds?: Coin[]) => Promise<ExecuteResult>;
  updateRoyaltyReceivers: ({
    receivers
  }: {
    receivers: WeightedAddress[];
  }, fee?: number | StdFee | "auto", memo?: string, _funds?: Coin[]) => Promise<ExecuteResult>;
  updateDenom: ({
    description,
    name,
    previewUri
  }: {
    description?: string;
    name?: string;
    previewUri?: string;
  }, fee?: number | StdFee | "auto", memo?: string, _funds?: Coin[]) => Promise<ExecuteResult>;
  purgeDenom: (fee?: number | StdFee | "auto", memo?: string, _funds?: Coin[]) => Promise<ExecuteResult>;
}
export class OmniflixOpenEditionMinterClient extends OmniflixOpenEditionMinterQueryClient implements OmniflixOpenEditionMinterInterface {
  client: SigningCosmWasmClient;
  sender: string;
  contractAddress: string;

  constructor(client: SigningCosmWasmClient, sender: string, contractAddress: string) {
    super(client, contractAddress);
    this.client = client;
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
  }

  mint = async (fee: number | StdFee | "auto" = "auto", memo?: string, _funds?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      mint: {}
    }, fee, memo, _funds);
  };
  mintAdmin = async ({
    recipient
  }: {
    recipient: string;
  }, fee: number | StdFee | "auto" = "auto", memo?: string, _funds?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      mint_admin: {
        recipient
      }
    }, fee, memo, _funds);
  };
  updateRoyaltyRatio = async ({
    ratio
  }: {
    ratio: string;
  }, fee: number | StdFee | "auto" = "auto", memo?: string, _funds?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      update_royalty_ratio: {
        ratio
      }
    }, fee, memo, _funds);
  };
  updateMintPrice = async ({
    mintPrice
  }: {
    mintPrice: Coin;
  }, fee: number | StdFee | "auto" = "auto", memo?: string, _funds?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      update_mint_price: {
        mint_price: mintPrice
      }
    }, fee, memo, _funds);
  };
  updateWhitelistAddress = async ({
    address
  }: {
    address: string;
  }, fee: number | StdFee | "auto" = "auto", memo?: string, _funds?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      update_whitelist_address: {
        address
      }
    }, fee, memo, _funds);
  };
  pause = async (fee: number | StdFee | "auto" = "auto", memo?: string, _funds?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      pause: {}
    }, fee, memo, _funds);
  };
  unpause = async (fee: number | StdFee | "auto" = "auto", memo?: string, _funds?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      unpause: {}
    }, fee, memo, _funds);
  };
  setPausers = async ({
    pausers
  }: {
    pausers: string[];
  }, fee: number | StdFee | "auto" = "auto", memo?: string, _funds?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      set_pausers: {
        pausers
      }
    }, fee, memo, _funds);
  };
  updateRoyaltyReceivers = async ({
    receivers
  }: {
    receivers: WeightedAddress[];
  }, fee: number | StdFee | "auto" = "auto", memo?: string, _funds?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      update_royalty_receivers: {
        receivers
      }
    }, fee, memo, _funds);
  };
  updateDenom = async ({
    description,
    name,
    previewUri
  }: {
    description?: string;
    name?: string;
    previewUri?: string;
  }, fee: number | StdFee | "auto" = "auto", memo?: string, _funds?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      update_denom: {
        description,
        name,
        preview_uri: previewUri
      }
    }, fee, memo, _funds);
  };
  purgeDenom = async (fee: number | StdFee | "auto" = "auto", memo?: string, _funds?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      purge_denom: {}
    }, fee, memo, _funds);
  };
}