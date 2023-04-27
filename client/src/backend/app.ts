import { l } from "../common/utils";
import { getSeed } from "./services/get-seed";
import { coin } from "@cosmjs/stargate";
import { INJ_DENOM } from "@injectivelabs/utils";
import { getCwHelpers } from "../common/helpers/cw-helpers";
import { fromDecimalToMicro, fromMicroToDecimal } from "../common/utils";
import {
  ATOM_CONTRACT,
  LUNA_CONTRACT,
  OSMO_CONTRACT,
  USDC_CONTRACT,
} from "../common/config/tokens.json";

async function main() {
  const seed = await getSeed();
  const {
    cwQueryConfig,
    cwQueryBalances,
    cwQueryPrices,
    cwUpdateConfig,
    cwUpdateToken,

    cwTransfer,
    cwInitTokens,
    cwQueryCw20Balances,
    owner,
  } = await getCwHelpers(seed);

  // await cwQueryConfig();
  // await cwUpdateConfig({ swapFeeRate: 0.003 });
  // await cwQueryConfig();

  // await cwQueryCw20Balances(owner);

  await cwQueryPrices();
  // await cwInitTokens();
  // await cwQueryPrices();
}

main();
