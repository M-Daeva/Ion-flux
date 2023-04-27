import { getCwHelpers } from "../helpers/cw-helpers-wallet";
import { initWalletList } from "../signers";
import { type ClientStructWithKeplr } from "../helpers/interfaces";
import { CONTRACT_ADDRESS, RPC, CHAIN_ID } from "../config/testnet-config.json";
import { INJ_DENOM } from "@injectivelabs/utils";
import chainRegistry from "../config/chain-registry.json";

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

  // user cosmwasm helpers
  const userCwHelpers = await getCwHelpers(userClientStruct, CONTRACT_ADDRESS);
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
