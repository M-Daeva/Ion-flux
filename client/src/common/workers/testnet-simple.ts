import { MsgBroadcaster } from "@injectivelabs/wallet-ts";
import {
  MsgSend,
  // Msgs,
  // MsgArg,
  createAnyMessage,
  toBase64,
} from "@injectivelabs/sdk-ts";
import { BigNumberInBase } from "@injectivelabs/utils";
import {
  WalletStrategy,
  type WalletStrategyArguments,
  Wallet,
} from "@injectivelabs/wallet-ts";
import { ChainId } from "@injectivelabs/ts-types";
import { Network } from "@injectivelabs/networks";
import { l } from "../utils";
import { getExecuteContractMsg } from "../signers/injective";
import { RPC, CONTRACT_ADDRESS } from "../config/testnet-config.json";
import { IonFluxMessageComposer as MessageComposer } from "../codegen/IonFlux.message-composer";
import { fromUtf8 } from "cosmwasm";
import {
  CosmWasmClient,
  type MsgExecuteContractEncodeObject,
} from "@cosmjs/cosmwasm-stargate";

import { tokenAddrToSymbolList, tokenInfoList } from "../helpers/general";
import TOKENS from "../config/tokens.json";
import { toUtf8 } from "@cosmjs/encoding";
import { UpdateConfigStruct } from "../helpers/interfaces";
import { IonFluxQueryClient as QueryClient } from "../codegen/IonFlux.client";
import { MsgExecuteContract } from "cosmjs-types/cosmwasm/wasm/v1/tx";

async function init() {
  const walletStrategyArguments: WalletStrategyArguments = {
    wallet: Wallet.Keplr,
    chainId: ChainId.Testnet,
  };

  const walletStrategy = new WalletStrategy(walletStrategyArguments);

  const msgBroadcastClient = new MsgBroadcaster({
    walletStrategy,
    network: Network.Testnet,
  });

  const signer: string = (await walletStrategy.getAddresses())[0];
  const composer = new MessageComposer(signer, CONTRACT_ADDRESS);

  const cwUnbond = async (tokenAddr: string, amount: number) => {
    try {
      const msgEncodeObject = composer.unbond({
        tokenAddr,
        amount: `${amount}`,
      });

      const msgArr = msgEncodeObject.value?.msg;
      if (!msgArr) throw new Error("Msg of msgEncodeObject is not found!");

      const _msg = JSON.parse(fromUtf8(msgArr));
      const msg = getExecuteContractMsg(CONTRACT_ADDRESS, signer, _msg);

      const tx = await msgBroadcastClient.broadcast({
        injectiveAddress: signer,
        msgs: msg,
      });

      return tx;
    } catch (error) {
      l(error);
    }
  };

  const cwDeposit = async (tokenAddr: string, amount: number) => {
    try {
      const contractMsg = { deposit: {} };

      const tokenMsg = {
        send: {
          contract: CONTRACT_ADDRESS,
          amount: `${amount}`,
          msg: toBase64(contractMsg),
        },
      };

      const msgExecuteContractEncodeObject: MsgExecuteContractEncodeObject = {
        typeUrl: "/cosmwasm.wasm.v1.MsgExecuteContract",
        value: MsgExecuteContract.fromPartial({
          sender: signer,
          contract: tokenAddr,
          msg: toUtf8(JSON.stringify(tokenMsg)),
          funds: undefined,
        }),
      };

      const msgArr = msgExecuteContractEncodeObject.value?.msg;
      if (!msgArr) throw new Error("Msg of msgEncodeObject is not found!");

      const _msg = JSON.parse(fromUtf8(msgArr));
      const msg = getExecuteContractMsg(CONTRACT_ADDRESS, signer, _msg);

      const tx = await msgBroadcastClient.broadcast({
        injectiveAddress: signer,
        msgs: msg,
      });

      return tx;
    } catch (error) {
      l(error);
    }
  };

  return {
    cwDeposit,
    cwUnbond,
  };
}

export { init };
