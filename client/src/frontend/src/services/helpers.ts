import type { DeliverTxResponse } from "@cosmjs/cosmwasm-stargate";
import type { TxResponse } from "@injectivelabs/sdk-ts";
import {
  txResStorage,
  isModalActiveStorage,
  LOCAL_STORAGE_KEY,
} from "../services/storage";

function displayModal(tx: DeliverTxResponse | TxResponse | undefined) {
  let txStatus: "Success" | "Error";
  let txHash: string;

  if (!tx) {
    txStatus = "Success";
    txHash = "";
  } else {
    txStatus = tx.rawLog.includes("failed") ? "Error" : "Success";

    if ("txHash" in tx) {
      txHash = tx.txHash;
    } else {
      txHash = tx.transactionHash;
    }
  }

  txResStorage.set([txStatus, txHash]);
  isModalActiveStorage.set(true);
}

function closeModal() {
  isModalActiveStorage.set(false);
}

function displayAddress() {
  const address = localStorage.getItem(LOCAL_STORAGE_KEY);
  if (!address || !address.includes("1")) {
    // displayModal("Connect wallet first!");
    return "";
  }

  const [prefix, ...[postfix]] = address.split("1");
  return `${prefix}...${postfix.slice(postfix.length - 4)}`;
}

// https://vitejs.dev/guide/assets.html#new-url-url-import-meta-url
function getImageUrl(name: string): string {
  return new URL(`/src/public/${name}`, import.meta.url).href;
}

export { displayModal, closeModal, displayAddress, getImageUrl };
