import { init } from "../../common/workers/testnet-backend-workers";
import { getSeed } from "./get-seed";
import { SEED_DAPP } from "../../common/config/testnet-config.json";

async function initContract() {
  const seed = await getSeed(SEED_DAPP);
  const { cwInitTokens } = await init(seed);
  await cwInitTokens();
}

initContract();
