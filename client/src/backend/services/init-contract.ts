import { init } from "../../common/workers/testnet-backend-workers";
import { getSeed } from "./get-seed";
import { SEED_DAPP } from "../../common/config/testnet-config.json";
import { tokenInfoList } from "../../common/helpers/general";
import { l } from "../../common/utils";

async function initContract() {
  const seed = await getSeed(SEED_DAPP);
  const { cwInitTokens, cwDeposit } = await init(seed);
  await cwInitTokens();

  for (const [tokenAddr, symbol, priceFeedStr] of tokenInfoList) {
    l(await cwDeposit(tokenAddr, 5_000 * 1e6));
  }
}

initContract();
