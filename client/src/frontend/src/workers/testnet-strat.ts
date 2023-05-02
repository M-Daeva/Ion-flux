import { getCwHelpers } from "./cw-helpers-strat";
import { initWalletList } from "../../../common/signers";
import { type ClientStructWithKeplr } from "../../../common/helpers/interfaces";
import {
  CONTRACT_ADDRESS,
  RPC,
  CHAIN_ID,
} from "../../../common/config/testnet-config.json";
import { INJ_DENOM } from "@injectivelabs/utils";
import chainRegistry from "../../../common/config/chain-registry.json";

import {
  WalletStrategy,
  type WalletStrategyArguments,
  Wallet,
} from "@injectivelabs/wallet-ts";
import { ChainId } from "@injectivelabs/ts-types";

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
