/**
* This file was automatically generated by @cosmwasm/ts-codegen@0.35.7.
* DO NOT MODIFY IT BY HAND. Instead, modify the source JSONSchema file,
* and run the @cosmwasm/ts-codegen generate command to regenerate this file.
*/

import { CosmWasmClient, SigningCosmWasmClient, ExecuteResult } from "@cosmjs/cosmwasm-stargate";
import { StdFee } from "@cosmjs/amino";
import { Addr, Timestamp, Uint64, Uint128, Decimal, InstantiateMsg, AuthDetails, CollectionDetails, WeightedAddress, OpenEditionMinterInitExtention, Coin, TokenDetails, ExecuteMsg, QueryMsg, OEMQueryExtension, Config, Uint32, Boolean, MintHistoryResponse, ArrayOfAddr, UserDetails, Token } from "./OmniflixOpenEditionMinter.types";
export interface OmniflixOpenEditionMinterReadOnlyInterface {
  contractAddress: string;
  collection: () => Promise<CollectionDetails>;
  tokenDetails: () => Promise<TokenDetails>;
  authDetails: () => Promise<AuthDetails>;
  config: () => Promise<Config>;
  userMintingDetails: ({
    address
  }: {
    address: string;
  }) => Promise<UserDetails>;
  isPaused: () => Promise<Boolean>;
  pausers: () => Promise<ArrayOfAddr>;
  extension: (oEMQueryExtension: OEMQueryExtension) => Promise<Uint32>;
  totalMintedCount: () => Promise<Uint32>;
  mintHistory: ({
    address
  }: {
    address: string;
  }) => Promise<MintHistoryResponse>;
}
export class OmniflixOpenEditionMinterQueryClient implements OmniflixOpenEditionMinterReadOnlyInterface {
  client: CosmWasmClient;
  contractAddress: string;

  constructor(client: CosmWasmClient, contractAddress: string) {
    this.client = client;
    this.contractAddress = contractAddress;
    this.collection = this.collection.bind(this);
    this.tokenDetails = this.tokenDetails.bind(this);
    this.authDetails = this.authDetails.bind(this);
    this.config = this.config.bind(this);
    this.userMintingDetails = this.userMintingDetails.bind(this);
    this.isPaused = this.isPaused.bind(this);
    this.pausers = this.pausers.bind(this);
    this.extension = this.extension.bind(this);
    this.totalMintedCount = this.totalMintedCount.bind(this);
    this.mintHistory = this.mintHistory.bind(this);
  }

  collection = async (): Promise<CollectionDetails> => {
    return this.client.queryContractSmart(this.contractAddress, {
      collection: {}
    });
  };
  tokenDetails = async (): Promise<TokenDetails> => {
    return this.client.queryContractSmart(this.contractAddress, {
      token_details: {}
    });
  };
  authDetails = async (): Promise<AuthDetails> => {
    return this.client.queryContractSmart(this.contractAddress, {
      auth_details: {}
    });
  };
  config = async (): Promise<Config> => {
    return this.client.queryContractSmart(this.contractAddress, {
      config: {}
    });
  };
  userMintingDetails = async ({
    address
  }: {
    address: string;
  }): Promise<UserDetails> => {
    return this.client.queryContractSmart(this.contractAddress, {
      user_minting_details: {
        address
      }
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
  extension = async (oEMQueryExtension: OEMQueryExtension): Promise<Uint32> => {
    return this.client.queryContractSmart(this.contractAddress, {
      extension: oEMQueryExtension
    });
  };
  totalMintedCount = async (): Promise<Uint32> => {
    return this.client.queryContractSmart(this.contractAddress, {
      total_minted_count: {}
    });
  };
  mintHistory = async ({
    address
  }: {
    address: string;
  }): Promise<MintHistoryResponse> => {
    return this.client.queryContractSmart(this.contractAddress, {
      mint_history: {
        address
      }
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
    collectionName,
    description,
    previewUri
  }: {
    collectionName?: string;
    description?: string;
    previewUri?: string;
  }, fee?: number | StdFee | "auto", memo?: string, _funds?: Coin[]) => Promise<ExecuteResult>;
  purgeDenom: (fee?: number | StdFee | "auto", memo?: string, _funds?: Coin[]) => Promise<ExecuteResult>;
  updateAdmin: ({
    admin
  }: {
    admin: string;
  }, fee?: number | StdFee | "auto", memo?: string, _funds?: Coin[]) => Promise<ExecuteResult>;
  updatePaymentCollector: ({
    paymentCollector
  }: {
    paymentCollector: string;
  }, fee?: number | StdFee | "auto", memo?: string, _funds?: Coin[]) => Promise<ExecuteResult>;
  burnRemainingTokens: (fee?: number | StdFee | "auto", memo?: string, _funds?: Coin[]) => Promise<ExecuteResult>;
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
    this.updateAdmin = this.updateAdmin.bind(this);
    this.updatePaymentCollector = this.updatePaymentCollector.bind(this);
    this.burnRemainingTokens = this.burnRemainingTokens.bind(this);
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
    collectionName,
    description,
    previewUri
  }: {
    collectionName?: string;
    description?: string;
    previewUri?: string;
  }, fee: number | StdFee | "auto" = "auto", memo?: string, _funds?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      update_denom: {
        collection_name: collectionName,
        description,
        preview_uri: previewUri
      }
    }, fee, memo, _funds);
  };
  purgeDenom = async (fee: number | StdFee | "auto" = "auto", memo?: string, _funds?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      purge_denom: {}
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
  updatePaymentCollector = async ({
    paymentCollector
  }: {
    paymentCollector: string;
  }, fee: number | StdFee | "auto" = "auto", memo?: string, _funds?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      update_payment_collector: {
        payment_collector: paymentCollector
      }
    }, fee, memo, _funds);
  };
  burnRemainingTokens = async (fee: number | StdFee | "auto" = "auto", memo?: string, _funds?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      burn_remaining_tokens: {}
    }, fee, memo, _funds);
  };
}