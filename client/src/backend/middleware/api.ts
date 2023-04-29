import { SEED_DAPP } from "../../common/config/testnet-config.json";
import { l, decrypt, specifyTimeout } from "../../common/utils";
import { getEncryptionKey } from "./key";
import { init } from "../../common/workers/testnet-backend-workers";
import TOKENS from "../../common/config/tokens.json";
import { TxResponse } from "@injectivelabs/sdk-ts";

async function transferTokens(recipient: string, tokenAddr: string) {
  try {
    const encryptionKey = getEncryptionKey();
    if (!encryptionKey) throw new Error("Key is not found!");

    const seed = decrypt(SEED_DAPP, encryptionKey);
    if (!seed) throw new Error("Key is wrong!");

    const helpers = await init(seed);
    if (!helpers) throw new Error("Init is failed!");

    const { cwTransfer } = helpers;

    const res: TxResponse = await specifyTimeout(
      cwTransfer(tokenAddr, 1_000_000_000, recipient)
    );
    l(res);
    return res;
  } catch (error) {
    l(error);
  }
}

export { transferTokens };
