import { init } from "../../common/workers/testnet-backend-workers";
import { getSeed } from "./get-seed";

async function initContract() {
  const seed = await getSeed();
  const { cwInitTokens } = await init(seed);
  await cwInitTokens();
}

initContract();
