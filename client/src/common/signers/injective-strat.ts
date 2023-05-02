import { l } from "../utils";
import { fromUtf8 } from "@cosmjs/encoding";
import { calculateFee as _calculateFee, GasPrice } from "@cosmjs/stargate";
import { type MsgExecuteContractEncodeObject } from "@cosmjs/cosmwasm-stargate";
import {
  Network,
  getNetworkEndpoints,
  getNetworkInfo,
} from "@injectivelabs/networks";
import {
  INJ_DENOM,
  BigNumberInBase,
  DEFAULT_BLOCK_TIMEOUT_HEIGHT,
  DEFAULT_GAS_PRICE,
  DEFAULT_STD_FEE,
} from "@injectivelabs/utils";
import {
  ChainRestAuthApi,
  ChainRestTendermintApi,
  BaseAccount,
  createTransaction,
  type CreateTransactionArgs,
  type Msgs,
  TxGrpcClient,
  type TxRaw,
  type TxResponse,
  MsgExecuteContract,
  type DirectSignResponse,
} from "@injectivelabs/sdk-ts";
import { WalletStrategy } from "@injectivelabs/wallet-ts";
import { ChainId } from "@injectivelabs/ts-types";

async function simulateFee(margin: number, gasPrice: string | GasPrice) {
  const gasWanted = Math.ceil(margin * +DEFAULT_STD_FEE.gas);

  return _calculateFee(gasWanted, gasPrice);
}

async function composeTxWithKeplr(
  walletStrategy: WalletStrategy,
  messages: Msgs[],
  margin: number,
  gasPrice: string | GasPrice,
  memo?: string
) {
  const injectiveAddress: string = (await walletStrategy.getAddresses())[0];
  const pubkey = await walletStrategy.getPubKey();

  const restEndpoint = getNetworkEndpoints(Network.Testnet);

  /** Account Details **/
  const chainRestAuthApi = new ChainRestAuthApi(restEndpoint.rest);
  const accountDetailsResponse = await chainRestAuthApi.fetchAccount(
    injectiveAddress
  );
  const baseAccount = BaseAccount.fromRestApi(accountDetailsResponse);

  /** Block Details */
  const chainRestTendermintApi = new ChainRestTendermintApi(restEndpoint.rest);
  const latestBlock = await chainRestTendermintApi.fetchLatestBlock();
  const latestHeight = latestBlock.header.height;
  const timeoutHeight = new BigNumberInBase(latestHeight).plus(
    DEFAULT_BLOCK_TIMEOUT_HEIGHT
  );

  /** Prepare the Transaction **/
  let txArgs: CreateTransactionArgs = {
    pubKey: pubkey,
    chainId: ChainId.Testnet,
    message: messages,
    sequence: baseAccount.sequence,
    // timeoutHeight: timeoutHeight.toNumber(),
    accountNumber: baseAccount.accountNumber,
  };

  const {
    txRaw: _txRaw,
    signBytes: _signBytes,
    signDoc: _signDoc,
  } = createTransaction(txArgs);

  txArgs.fee = await simulateFee(margin, gasPrice);

  const { txRaw, signBytes, signDoc } = createTransaction(txArgs);

  return { txRaw, signBytes, signDoc, txArgs };
}

async function signTxWithKeplr(
  walletStrategy: WalletStrategy,
  txArgs: CreateTransactionArgs,
  txRaw: TxRaw
) {
  const address: string = (await walletStrategy.getAddresses())[0];
  const { chainId, accountNumber } = txArgs;

  const response = await walletStrategy.signCosmosTransaction({
    txRaw,
    accountNumber,
    chainId,
    address,
  });

  return response;
}

async function broadcastTxWithKeplr(
  walletStrategy: WalletStrategy,
  tx: TxRaw | DirectSignResponse
) {
  const injectiveAddress: string = (await walletStrategy.getAddresses())[0];

  const txResponse = await walletStrategy.sendTransaction(tx, {
    address: injectiveAddress,
    chainId: ChainId.Testnet,
  });

  return txResponse;
}

function signAndBroadcastWrapperWithKeplr(
  walletStrategy: WalletStrategy,
  margin: number,
  gasPrice: string | GasPrice
) {
  return async (messages: Msgs[], memo?: string): Promise<TxResponse> => {
    const { txRaw, txArgs } = await composeTxWithKeplr(
      walletStrategy,
      messages,
      margin,
      gasPrice,
      memo
    );

    const signed = await signTxWithKeplr(walletStrategy, txArgs, txRaw);

    const txResponse = await broadcastTxWithKeplr(walletStrategy, signed);

    return txResponse;
  };
}

function getExecuteContractMsg(
  contractAddress: string,
  sender: string,
  msg: object,
  funds?:
    | {
        denom: string;
        amount: string;
      }
    | {
        denom: string;
        amount: string;
      }[]
) {
  return new MsgExecuteContract({
    contractAddress,
    sender,
    msg,
    funds,
  });
}

async function initWithKeplr(
  contractAddress: string,
  walletStrategy: WalletStrategy,
  margin: number = 2,
  gasPrice: string | GasPrice = `${DEFAULT_GAS_PRICE}${INJ_DENOM}`
) {
  const injectiveAddress: string = (await walletStrategy.getAddresses())[0];

  const signAndBroadcast = signAndBroadcastWrapperWithKeplr(
    walletStrategy,
    margin,
    gasPrice
  );

  const executeContract = async (
    msg: object,
    funds?: {
      denom: string;
      amount: string;
    }[]
  ) => {
    return await signAndBroadcast([
      getExecuteContractMsg(contractAddress, injectiveAddress, msg, funds),
    ]);
  };

  const execWrapper = async (
    msgEncodeObject: MsgExecuteContractEncodeObject,
    funds?: {
      denom: string;
      amount: string;
    }[]
  ) => {
    try {
      const msgArr = msgEncodeObject.value?.msg;
      if (!msgArr) throw new Error("Msg of msgEncodeObject is not found!");

      const msg = JSON.parse(fromUtf8(msgArr));
      const tx = await executeContract(msg, funds);
      // const { txHash, gasWanted, gasUsed, rawLog } = tx;
      // l("\n", { txHash, gasWanted, gasUsed, rawLog }, "\n");
      return tx;
    } catch (error) {
      l("\n", error, "\n");
    }
  };

  return {
    injectiveAddress,
    signAndBroadcast,
    executeContract,
    execWrapper,
  };
}

export { initWithKeplr };
