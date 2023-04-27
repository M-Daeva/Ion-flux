import { SEED_DAPP } from "../../common/config/testnet-config.json";
import { l, decrypt } from "../../common/utils";
import { getEncryptionKey } from "./key";
import { init } from "../../common/workers/testnet-backend-workers";
import TOKENS from "../../common/config/tokens.json";
import { TxResponse } from "@injectivelabs/sdk-ts";

async function transferTokens(recipient: string) {
  try {
    const encryptionKey = getEncryptionKey();
    if (!encryptionKey) throw new Error("Key is not found!");

    const seed = decrypt(SEED_DAPP, encryptionKey);
    if (!seed) throw new Error("Key is wrong!");

    const helpers = await init(seed);
    if (!helpers) throw new Error("Init is failed!");

    const { cwTransfer } = helpers;

    let promiseList: Promise<TxResponse | undefined>[] = [];

    for (const [k, v] of Object.entries(TOKENS)) {
      if (k === "CONTRACT_CODE") continue;

      promiseList.push(cwTransfer(v, 1_000_000_000, recipient));
    }

    await Promise.all(promiseList);

    return { fn: "transferTokens", isOk: true };
  } catch (error) {
    l(error);

    return { fn: "transferTokens", isOk: false };
  }
}

export { transferTokens };
