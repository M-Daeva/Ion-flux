import { getCwHelpers } from "../helpers/cw-helpers-strat";
import { initWalletList } from "../signers";
import { type ClientStructWithKeplr } from "../helpers/interfaces";
import { CONTRACT_ADDRESS, RPC, CHAIN_ID } from "../config/testnet-config.json";
import { INJ_DENOM } from "@injectivelabs/utils";
import chainRegistry from "../config/chain-registry.json";

import {
  WalletStrategy,
  type WalletStrategyArguments,
  Wallet,
} from "@injectivelabs/wallet-ts";
import { ChainId } from "@injectivelabs/ts-types";
import {
  chainInfos,
  getNetworkChainInfo,
  getNetworkInfo,
  Network,
  getNetworkEndpoints,
} from "@injectivelabs/networks";
import type {
  Keplr,
  Window as KeplrWindow,
  ChainInfo,
} from "@keplr-wallet/types";

declare global {
  interface Window extends KeplrWindow {}
}

async function init() {
  if (!chainRegistry) return;

  const chainType: "main" | "test" = "test";

  const chain = chainRegistry.find((item) => item.denomNative === INJ_DENOM);
  if (!chain) return;

  const wallet = await initWalletList([chain], chainType);
  if (!wallet) return;

  const userClientStruct: ClientStructWithKeplr = {
    RPC,
    wallet,
    chainId: CHAIN_ID,
  };

  //-------------

  // const { keplr } = window;
  // if (!keplr) throw new Error("You need to install Keplr");

  // const chainInfo = getNetworkChainInfo(Network.Testnet);
  // const endPoints = getNetworkEndpoints(Network.Testnet);

  // await keplr.experimentalSuggestChain({ bech32Config });

  // await _addChainList(wallet, chainRegistry, chainType); // add network to Keplr
  // await _unlockWalletList(wallet, chainRegistry, chainType); // give permission for the webpage to access Keplr
  // return wallet;

  //--------------------

  const walletStrategyArguments: WalletStrategyArguments = {
    wallet: Wallet.Keplr,
    chainId: ChainId.Testnet,
  };

  const walletStrategy = new WalletStrategy(walletStrategyArguments);

  // user cosmwasm helpers
  const userCwHelpers = await getCwHelpers(walletStrategy, userClientStruct);
  if (!userCwHelpers) return;

  const {
    owner,

    cwDeposit,
    cwSwap,

    cwUpdateConfig,
    cwUpdateToken,
    cwUnbond,
    cwWithdraw,
    cwClaim,
    cwSwapAndClaim,

    cwQueryConfig,
    cwQueryTokensWeight,
    cwQueryLiquidity,
    cwQueryProviders,
    cwQueryTokens,
    cwQueryBalances,
    cwQueryPrices,

    cwQueryCw20Balances,
  } = userCwHelpers;

  return {
    owner,

    cwDeposit,
    cwSwap,

    cwUnbond,
    cwWithdraw,
    cwClaim,
    cwSwapAndClaim,

    cwQueryConfig,
    cwQueryTokensWeight,
    cwQueryLiquidity,
    cwQueryProviders,
    cwQueryTokens,
    cwQueryPrices,

    cwQueryCw20Balances,
  };
}

export { init };
