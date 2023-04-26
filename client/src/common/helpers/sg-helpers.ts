import { l } from "../utils";
import { Coin } from "@cosmjs/stargate";
import { init } from "../signers/injective";
import { Network, getNetworkEndpoints } from "@injectivelabs/networks";
import {
  MsgSend,
  ChainRestBankApi,
  IndexerGrpcOracleApi,
} from "@injectivelabs/sdk-ts";

async function getSgHelpers(seed: string) {
  const { injectiveAddress, signAndBroadcast } = await init(seed);

  async function sgSend(recipient: string, amount: Coin) {
    try {
      const msg = new MsgSend({
        srcInjectiveAddress: injectiveAddress,
        dstInjectiveAddress: recipient,
        amount,
      });

      const tx = await signAndBroadcast([msg]);
      const { txHash, gasWanted, gasUsed, rawLog } = tx;
      l("\n", { txHash, gasWanted, gasUsed, rawLog }, "\n");
    } catch (error) {
      l("\n", error, "\n");
    }
  }

  async function sgQueryBalances(address: string) {
    try {
      const endpoints = getNetworkEndpoints(Network.Testnet);
      const chainRestBankApi = new ChainRestBankApi(endpoints.rest);
      const { balances: res } = await chainRestBankApi.fetchBalances(address);
      // l("\n", res, "\n");
      return res;
    } catch (error) {
      l("\n", error, "\n");
    }
  }

  async function sgQueryOracle(baseSymbol: string) {
    try {
      const endpoints = getNetworkEndpoints(Network.TestnetK8s);
      const indexerGrpcOracleApi = new IndexerGrpcOracleApi(endpoints.indexer);

      const quoteSymbol = "USDT";
      const oracleType = "bandibc"; // primary oracle we use

      const res = await indexerGrpcOracleApi.fetchOraclePriceNoThrow({
        baseSymbol,
        quoteSymbol,
        oracleType,
      });

      l("\n", res, "\n");
      return res;
    } catch (error) {
      l("\n", error, "\n");
    }
  }

  return {
    owner: injectiveAddress,
    sgSend,
    sgQueryBalances,
    sgQueryOracle,
  };
}

export { getSgHelpers };
