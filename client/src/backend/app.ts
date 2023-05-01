import { l } from "../common/utils";
import { getSeed } from "./services/get-seed";
import { coin } from "@cosmjs/stargate";
import { INJ_DENOM } from "@injectivelabs/utils";
import { init } from "../common/workers/testnet-backend-workers";
import { fromDecimalToMicro, fromMicroToDecimal } from "../common/utils";
import {
  SEED_ALICE,
  SEED_DAPP,
  CONTRACT_ADDRESS,
} from "../common/config/testnet-config.json";
import {
  ATOM_CONTRACT,
  LUNA_CONTRACT,
  OSMO_CONTRACT,
  USDC_CONTRACT,
} from "../common/config/tokens.json";

async function main() {
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
  } = await init(await getSeed(SEED_DAPP));

  // l(await cwQueryConfig());
  // l(await cwUpdateConfig({ priceAge: 8000000 }));
  // await cwQueryConfig();

  // l(
  //   await cwTransfer(
  //     ATOM_CONTRACT,
  //     1e6,
  //     "inj1amp7dv5fvjyx95ea4grld6jmu9v207awtefwce"
  //   )
  // );
  // l(await cwDeposit(ATOM_CONTRACT, 110 * 1e6));
  // l(await cwDeposit(USDC_CONTRACT, 120 * 1e6));

  l((await cwQueryProviders())?.[0]);

  //l(await cwQueryCw20Balances(owner));

  // l(await cwQueryPrices());
  // await cwInitTokens();
  // await cwQueryPrices();
}

main();
