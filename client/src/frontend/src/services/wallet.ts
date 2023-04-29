import { init as _init } from "../../../common/workers/testnet-frontend-workers";
import { displayModal } from "./helpers";
import { addressStorage, LOCAL_STORAGE_KEY } from "../services/storage";
import { l } from "../../../common/utils";

async function init() {
  // init wallet, add inj chain, save address to localSorage
  async function initCwHandler() {
    try {
      const { owner: address } = await _init();
      addressStorage.set(address);
      localStorage.setItem(LOCAL_STORAGE_KEY, address);
      window.location.reload();
    } catch (error) {
      displayModal(error);
    }
  }

  return {
    initCwHandler,
  };
}

export { init };
