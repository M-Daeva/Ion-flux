import { initStorage } from "../storages";
import { EncryptionKeyStorage } from "../../common/helpers/interfaces";
import { decrypt } from "../../common/utils";
import { init } from "../../common/signers/injective";
import {
  SEED_DAPP,
  CONTRACT_ADDRESS,
} from "../../common/config/testnet-config.json";
import { DAPP_ADDRESS } from "../envs";

let _encryptionKeyStorage = initStorage<EncryptionKeyStorage>(
  "encryption-key-storage"
);

function getEncryptionKey() {
  return _encryptionKeyStorage.get();
}

async function setEncryptionKey(value: string): Promise<string> {
  try {
    // skip if key specified
    if (_encryptionKeyStorage.get()) {
      throw new Error(`Key is already specified!`);
    }

    // skip if key is wrong
    const seed = decrypt(SEED_DAPP, value);
    if (!seed) throw new Error(`Key '${value}' is wrong!`);

    const { injectiveAddress } = await init(CONTRACT_ADDRESS, seed);

    if (injectiveAddress !== DAPP_ADDRESS)
      throw new Error(`Key '${value}' is wrong!`);

    _encryptionKeyStorage.set(value);

    return "Success!";
  } catch (error) {
    return `${error}`;
  }
}

export { getEncryptionKey, setEncryptionKey };
