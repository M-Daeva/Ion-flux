import { l } from "../utils";
import { init } from "../signers/injective";
import { CONTRACT_ADDRESS, RPC } from "../config/testnet-config.json";
import TOKENS from "../config/tokens.json";
import { fromUtf8, toUtf8 } from "@cosmjs/encoding";
import { MsgExecuteContract } from "cosmjs-types/cosmwasm/wasm/v1/tx";
import { UpdateConfigStruct } from "./interfaces";
import { IonFluxQueryClient as QueryClient } from "../codegen/IonFlux.client";
import { IonFluxMessageComposer as MessageComposer } from "../codegen/IonFlux.message-composer";
import {
  CosmWasmClient,
  MsgExecuteContractEncodeObject,
} from "@cosmjs/cosmwasm-stargate";

const _toStr = (n?: number): string | undefined => (n ? `${n}` : undefined);

async function getCwHelpers(seed: string) {
  const { executeContract, injectiveAddress } = await init(seed);

  const composer = new MessageComposer(injectiveAddress, CONTRACT_ADDRESS);
  const cosmwasmClient = await CosmWasmClient.connect(RPC);
  const queryClient = new QueryClient(cosmwasmClient, CONTRACT_ADDRESS);

  const _execWrapper = async (
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
      const { txHash, gasWanted, gasUsed, rawLog } = tx;
      l("\n", { txHash, gasWanted, gasUsed, rawLog }, "\n");
      return tx;
    } catch (error) {
      l("\n", error, "\n");
    }
  };

  async function cwDeposit(tokenAddr: string, amount: number) {
    let contractMsg = { deposit: {} };

    let tokenMsg = {
      send: {
        contract: CONTRACT_ADDRESS,
        amount: `${amount}`,
        msg: toUtf8(JSON.stringify(contractMsg)),
      },
    };

    let msgExecuteContractEncodeObject: MsgExecuteContractEncodeObject = {
      typeUrl: "/cosmwasm.wasm.v1.MsgExecuteContract",
      value: MsgExecuteContract.fromPartial({
        sender: injectiveAddress,
        contract: tokenAddr,
        msg: toUtf8(JSON.stringify(tokenMsg)),
        funds: undefined,
      }),
    };

    return await _execWrapper(msgExecuteContractEncodeObject);
  }

  async function cwSwap(
    tokenAddr: string,
    amount: number,
    tokenOutAddr: string
  ) {
    let contractMsg = {
      swap: {
        token_out_addr: tokenOutAddr,
      },
    };

    let tokenMsg = {
      send: {
        contract: CONTRACT_ADDRESS,
        amount: `${amount}`,
        msg: toUtf8(JSON.stringify(contractMsg)),
      },
    };

    let msgExecuteContractEncodeObject: MsgExecuteContractEncodeObject = {
      typeUrl: "/cosmwasm.wasm.v1.MsgExecuteContract",
      value: MsgExecuteContract.fromPartial({
        sender: injectiveAddress,
        contract: tokenAddr,
        msg: toUtf8(JSON.stringify(tokenMsg)),
        funds: undefined,
      }),
    };

    return await _execWrapper(msgExecuteContractEncodeObject);
  }

  async function cwUpdateConfig(updateConfigStruct: UpdateConfigStruct) {
    const { admin, swapFeeRate, window, unbondingPeriod, priceAge } =
      updateConfigStruct;

    return await _execWrapper(
      composer.updateConfig({
        admin,
        swapFeeRate: _toStr(swapFeeRate),
        window: _toStr(window),
        unbondingPeriod: _toStr(unbondingPeriod),
        priceAge: _toStr(priceAge),
      })
    );
  }

  async function cwUpdateToken(
    tokenAddr: string,
    symbol: string,
    priceFeedIdStr: string
  ) {
    return await _execWrapper(
      composer.updateToken({ tokenAddr, symbol, priceFeedIdStr })
    );
  }

  async function cwUnbond(tokenAddr: string, amount: number) {
    return await _execWrapper(
      composer.unbond({ tokenAddr, amount: `${amount}` })
    );
  }

  async function cwWithdraw(tokenAddr: string, amount: number) {
    return await _execWrapper(
      composer.withdraw({ tokenAddr, amount: `${amount}` })
    );
  }

  async function cwClaim() {
    return await _execWrapper(composer.claim());
  }

  async function cwSwapAndClaim(tokenOutAddr: string) {
    return await _execWrapper(composer.swapAndClaim({ tokenOutAddr }));
  }

  async function cwQueryConfig() {
    try {
      const res = await queryClient.queryConfig();
      l("\n", res, "\n");
      return res;
    } catch (error) {
      l("\n", error, "\n");
    }
  }

  async function cwQueryTokensWeight(addressList: string[] = []) {
    try {
      const res = await queryClient.queryTokensWeight({ addressList });
      l("\n", res, "\n");
      return res;
    } catch (error) {
      l("\n", error, "\n");
    }
  }

  async function cwQueryLiquidity(addressList: string[] = []) {
    try {
      const res = await queryClient.queryLiquidity({ addressList });
      l("\n", res, "\n");
      return res;
    } catch (error) {
      l("\n", error, "\n");
    }
  }

  async function cwQueryProviders(addressList: string[] = []) {
    try {
      const res = await queryClient.queryProviders({ addressList });
      l("\n", res, "\n");
      return res;
    } catch (error) {
      l("\n", error, "\n");
    }
  }

  async function cwQueryTokens(addressList: string[] = []) {
    try {
      const res = await queryClient.queryTokens({ addressList });
      l("\n", res, "\n");
      return res;
    } catch (error) {
      l("\n", error, "\n");
    }
  }

  async function cwQueryBalances(addressList: string[] = []) {
    try {
      // let res0 = await queryClient.queryBalances({ addressList });
      // let res = _tokenAddrToSymbolList(
      //   res0.map(({ amount, token_addr }) => [token_addr, amount])
      // );
      let res = await queryClient.queryBalances({ addressList });
      l("\n", res, "\n");
      return res;
    } catch (error) {
      l("\n", error, "\n");
    }
  }

  async function cwQueryPrices(addressList: string[] = []) {
    try {
      const res0 = await queryClient.queryPrices({ addressList });
      let res = _tokenAddrToSymbolList(res0);
      l("\n", res, "\n");
      return res;
    } catch (error) {
      l("\n", error, "\n");
    }
  }

  // ------------ custom functions ---------------

  // for faucet
  async function cwTransfer(
    tokenAddr: string,
    amount: number,
    recipient: string
  ) {
    let tokenMsg = {
      transfer: {
        recipient,
        amount: `${amount}`,
      },
    };

    let msgExecuteContractEncodeObject: MsgExecuteContractEncodeObject = {
      typeUrl: "/cosmwasm.wasm.v1.MsgExecuteContract",
      value: MsgExecuteContract.fromPartial({
        sender: injectiveAddress,
        contract: tokenAddr,
        msg: toUtf8(JSON.stringify(tokenMsg)),
        funds: undefined,
      }),
    };

    return await _execWrapper(msgExecuteContractEncodeObject);
  }

  async function cwInitTokens() {
    let config = [
      [
        TOKENS.ATOM_CONTRACT,
        "ATOM",
        "0x61226d39beea19d334f17c2febce27e12646d84675924ebb02b9cdaea68727e3",
      ],
      [
        TOKENS.LUNA_CONTRACT,
        "LUNA",
        "0x677dbbf4f68b5cb996a40dfae338b87d5efb2e12a9b2686d1ca16d69b3d7f204",
      ],
      [
        TOKENS.USDC_CONTRACT,
        "USDC",
        "0x41f3625971ca2ed2263e78573fe5ce23e13d2558ed3f2e47ab0f84fb9e7ae722",
      ],
      [
        TOKENS.OSMO_CONTRACT,
        "OSMO",
        "0xd9437c194a4b00ba9d7652cd9af3905e73ee15a2ca4152ac1f8d430cc322b857",
      ],
    ];

    for (const [tokenAddr, symbol, priceFeedIdStr] of config) {
      await cwUpdateToken(tokenAddr, symbol, priceFeedIdStr);
    }
  }

  async function cwQueryCw20Balances(wallet: string) {
    const tokens: [string, string][] = Object.entries(TOKENS);
    let balanceList: [string, number][] = [];

    const promiseList = tokens.map(async ([k, v]) => {
      if (k === "CONTRACT_CODE") return;

      try {
        const res: { balance: string } =
          await queryClient.client.queryContractSmart(v, {
            balance: { address: wallet },
          });

        balanceList.push([v, +res.balance / 1e6]);
      } catch (error) {}
    });

    await Promise.all(promiseList);

    balanceList = _tokenAddrToSymbolList(balanceList);

    l("\n", balanceList, "\n");

    return balanceList;
  }

  function _tokenAddrToSymbolList(addrAndValueList: [string, any][]) {
    const tokens: [string, string][] = Object.entries(TOKENS);
    let res: [string, any][] = [];

    for (const [addr, value] of addrAndValueList) {
      let token = tokens.find(([k, v]) => v === addr);
      let symbol = token?.[0].split("_")[0];
      if (!symbol) continue;

      res.push([symbol, value]);
    }

    return res.sort((a, b) => (a[0] >= b[0] ? 1 : -1));
  }

  return {
    owner: injectiveAddress,

    cwDeposit,
    cwSwap,

    cwUpdateConfig,
    cwUpdateToken,
    cwUnbond,
    cwWithdraw,
    cwClaim,
    cwSwapAndClaim,

    cwQueryConfig,
    cwQueryTokensWeight,
    cwQueryLiquidity,
    cwQueryProviders,
    cwQueryTokens,
    cwQueryBalances,
    cwQueryPrices,

    cwTransfer,
    cwInitTokens,
    cwQueryCw20Balances,
  };
}

export { getCwHelpers };
