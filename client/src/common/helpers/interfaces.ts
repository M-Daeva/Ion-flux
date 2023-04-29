import { Coin } from "@cosmjs/stargate";
import { Keplr } from "@keplr-wallet/types";

interface UpdateConfigStruct {
  admin?: string;
  swapFeeRate?: number;
  window?: number;
  unbondingPeriod?: number;
  priceAge?: number;
}

interface ClientStructWithKeplr {
  RPC: string;
  wallet: Keplr;
  chainId: string;
}

interface ClientStructWithoutKeplr {
  RPC: string;
  seed: string;
  prefix: string;
}

type ClientStruct = ClientStructWithKeplr | ClientStructWithoutKeplr;

interface ChainsResponse {
  [chains: string]: string[];
}

interface ChainResponse {
  $schema: string;
  chain_name: string;
  status: string;
  network_type: string;
  pretty_name: string;
  chain_id: string;
  bech32_prefix: string;
  daemon_name: string;
  node_home: string;
  genesis?: {
    genesis_url: string;
  };
  key_algos?: string[];
  slip44: number;
  fees: {
    fee_tokens: {
      denom: string;
      fixed_min_gas_price?: number;
      low_gas_price?: number;
      average_gas_price?: number;
      high_gas_price?: number;
    }[];
  };
  codebase?: {
    git_repo: string;
    recommended_version: string;
    compatible_versions: string[];
    binaries: {
      "linux/amd64": string;
      "darwin/amd64": string;
    };
    genesis: {
      genesis_url: string;
    };
  };
  peers: {
    seeds: {
      id: string;
      address: string;
    }[];
    persistent_peers: {
      id: string;
      address: string;
      provider: string;
    }[];
  };
  apis: {
    rpc: {
      address: string;
      provider: string;
    }[];
    rest: {
      address: string;
      provider: string;
    }[];
    grpc: {
      address: string;
      provider: string;
    }[];
  };
  explorers: {
    kind: string;
    url: string;
    tx_page: string;
  }[];
}

interface NetworkData {
  prefix: string;
  main?: ChainResponse;
  test?: ChainResponse;
  img: string;
  symbol: string;
  exponent: number;
  denomNative: string;
  denomIbc: string;
  coinGeckoId?: string;
}

interface AssetList {
  $schema: string;
  chain_name: string;
  assets: {
    description: string;
    denom_units: {
      denom: string;
      exponent: number;
      aliases: string[];
    }[];
    base: string;
    name: string;
    display: string;
    symbol: string;
    logo_URIs: {
      png: string;
      svg?: string;
    };
    coingecko_id: string;
    keywords: string[];
  }[];
}

interface NetworkContentResponse {
  name: string;
  path: string;
  sha: string;
  size: number;
  url: string;
  html_url: string;
  git_url: string;
  download_url: null;
  type: string;
  _links: {
    self: string;
    git: string;
    html: string;
  };
}

interface TimeInHoursAndMins {
  hours: number;
  minutes: number;
}

type StorageNames = "encryption-key-storage";

type EncryptionKeyStorage = string;

type StorageTypes = EncryptionKeyStorage;

export type { NetworkData, ClientStructWithKeplr, ClientStructWithoutKeplr };

export {
  AssetList,
  ClientStruct,
  ChainsResponse,
  ChainResponse,
  NetworkContentResponse,
  StorageNames,
  EncryptionKeyStorage,
  StorageTypes,
  TimeInHoursAndMins,
  UpdateConfigStruct,
};
