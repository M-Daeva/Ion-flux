import { specifyTimeout as _specifyTimeout } from "../utils";
import { getCwHelpers } from "../helpers/cw-helpers";

async function init(seed: string) {
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

    cwTransfer,
    cwInitTokens,
    cwQueryCw20Balances,
  } = await getCwHelpers(seed);

  return {
    owner,

    cwUpdateConfig,

    cwTransfer,
    cwInitTokens,
  };
}

export { init };
