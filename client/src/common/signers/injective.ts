import { l } from "../utils";
import { calculateFee as _calculateFee, GasPrice } from "@cosmjs/stargate";
import { ChainId } from "@injectivelabs/ts-types";
import { CONTRACT_ADDRESS } from "../config/testnet-config.json";
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
} from "@injectivelabs/utils";
import {
  PrivateKey,
  ChainRestAuthApi,
  ChainRestTendermintApi,
  BaseAccount,
  createTransaction,
  CreateTransactionArgs,
  Msgs,
  TxClient,
  TxGrpcClient,
  TxRaw,
  TxResponse,
  MsgExecuteContract,
} from "@injectivelabs/sdk-ts";

const ethereumDerivationPath = "m/44'/60'/0'/0/0"; // slip44: 60
const cosmosDerivationPath = "m/44'/118'/0'/0/0"; // slip44: 118

function getPrivateKey(seed: string) {
  const privateKey = PrivateKey.fromMnemonic(seed, ethereumDerivationPath);
  const injectiveAddress = privateKey.toAddress().bech32Address;

  return { privateKey, injectiveAddress };
}

async function simulateFee(
  txRaw: TxRaw,
  signature: Uint8Array,
  margin: number,
  gasPrice: string | GasPrice
) {
  const network = getNetworkInfo(Network.TestnetK8s);
  txRaw.signatures = [signature];
  const txService = new TxGrpcClient(network.grpc);

  const gasSimulated = (await txService.simulate(txRaw)).gasInfo.gasUsed;
  const gasWanted = Math.ceil(margin * gasSimulated);

  return _calculateFee(gasWanted, gasPrice);
}

async function composeTx(
  privateKey: PrivateKey,
  messages: Msgs[],
  margin: number,
  gasPrice: string | GasPrice,
  memo?: string
) {
  const injectiveAddress = privateKey.toAddress().bech32Address;
  const pubKey = privateKey.toPublicKey().toBase64();
  const chainId = ChainId.Testnet;
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
    pubKey,
    chainId,
    message: messages,
    sequence: baseAccount.sequence,
    timeoutHeight: timeoutHeight.toNumber(),
    accountNumber: baseAccount.accountNumber,
    memo,
  };
  const { txRaw: _txRaw, signBytes: _signBytes } = createTransaction(txArgs);

  const _signature = await signTx(privateKey, _signBytes);
  txArgs.fee = await simulateFee(_txRaw, _signature, margin, gasPrice);
  const { txRaw, signBytes } = createTransaction(txArgs);

  return { txRaw, signBytes };
}

async function signTx(privateKey: PrivateKey, signBytes: Uint8Array) {
  const signature = await privateKey.sign(Buffer.from(signBytes));
  return signature;
}

async function broadcastTx(txRaw: TxRaw, signature: Uint8Array) {
  /** Append Signatures */
  const network = getNetworkInfo(Network.TestnetK8s);
  txRaw.signatures = [signature];

  /** Calculate hash of the transaction */
  // l(`Transaction Hash: ${TxClient.hash(txRaw)}`);

  const txService = new TxGrpcClient(network.grpc);

  /** Broadcast transaction */
  const txResponse = await txService.broadcast(txRaw);

  return txResponse;
}

function signAndBroadcastWrapper(
  seed: string,
  margin: number,
  gasPrice: string | GasPrice
) {
  return async (messages: Msgs[], memo?: string): Promise<TxResponse> => {
    const { privateKey } = getPrivateKey(seed);
    const { txRaw, signBytes } = await composeTx(
      privateKey,
      messages,
      margin,
      gasPrice,
      memo
    );
    const signature = await signTx(privateKey, signBytes);

    return await broadcastTx(txRaw, signature);
  };
}

function getExecuteContractMsg(
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
    contractAddress: CONTRACT_ADDRESS,
    sender,
    msg,
    funds,
  });
}

async function init(
  seed: string,
  margin: number = 1.2,
  gasPrice: string | GasPrice = `${DEFAULT_GAS_PRICE}${INJ_DENOM}`
) {
  const { injectiveAddress, privateKey } = getPrivateKey(seed);
  const signAndBroadcast = signAndBroadcastWrapper(seed, margin, gasPrice);

  const executeContract = async (
    msg: object,
    funds?: {
      denom: string;
      amount: string;
    }[]
  ) => {
    return await signAndBroadcast([
      getExecuteContractMsg(injectiveAddress, msg, funds),
    ]);
  };

  return { privateKey, injectiveAddress, signAndBroadcast, executeContract };
}

export { init };
