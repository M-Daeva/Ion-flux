import { access, readFile } from "fs/promises";
import { rootPath, decrypt, l } from "../../common/utils";
import { SEED_DAPP } from "../../common/config/testnet-config.json";

async function getSeed(): Promise<string> {
  const keyPath = rootPath("../../.test-wallets/key");

  try {
    await access(keyPath);
    const encryptionKey = await readFile(keyPath, { encoding: "utf-8" });
    const seed = decrypt(SEED_DAPP, encryptionKey);
    if (!seed) throw new Error("Can not get seed!");
    return seed;
  } catch (error) {
    l(error);
    return "";
  }
}

export { getSeed };
