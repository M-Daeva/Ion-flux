import { l } from "../utils";
import { tokenAddrToSymbolList } from "./general";
import { init } from "../signers/injective";
import { CONTRACT_ADDRESS, RPC } from "../config/testnet-config.json";
import TOKENS from "../config/tokens.json";
import { toUtf8 } from "@cosmjs/encoding";
import { MsgExecuteContract } from "cosmjs-types/cosmwasm/wasm/v1/tx";
import { UpdateConfigStruct } from "./interfaces";
import { IonFluxQueryClient as QueryClient } from "../codegen/IonFlux.client";
import { IonFluxMessageComposer as MessageComposer } from "../codegen/IonFlux.message-composer";
import { tokenInfoList } from "./general";
import {
  CosmWasmClient,
  MsgExecuteContractEncodeObject,
} from "@cosmjs/cosmwasm-stargate";

const _toStr = (n?: number): string | undefined => (n ? `${n}` : undefined);

async function getCwHelpers(seed: string) {
  const { injectiveAddress, execWrapper } = await init(CONTRACT_ADDRESS, seed);

  const composer = new MessageComposer(injectiveAddress, CONTRACT_ADDRESS);
  const cosmwasmClient = await CosmWasmClient.connect(RPC);
  const queryClient = new QueryClient(cosmwasmClient, CONTRACT_ADDRESS);

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

    return await execWrapper(msgExecuteContractEncodeObject);
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

    return await execWrapper(msgExecuteContractEncodeObject);
  }

  async function cwUpdateConfig(updateConfigStruct: UpdateConfigStruct) {
    const { admin, swapFeeRate, window, unbondingPeriod, priceAge } =
      updateConfigStruct;

    return await execWrapper(
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
    return await execWrapper(
      composer.updateToken({ tokenAddr, symbol, priceFeedIdStr })
    );
  }

  async function cwUnbond(tokenAddr: string, amount: number) {
    return await execWrapper(
      composer.unbond({ tokenAddr, amount: `${amount}` })
    );
  }

  async function cwWithdraw(tokenAddr: string, amount: number) {
    return await execWrapper(
      composer.withdraw({ tokenAddr, amount: `${amount}` })
    );
  }

  async function cwClaim() {
    return await execWrapper(composer.claim());
  }

  async function cwSwapAndClaim(tokenOutAddr: string) {
    return await execWrapper(composer.swapAndClaim({ tokenOutAddr }));
  }

  async function cwQueryConfig() {
    try {
      return await queryClient.queryConfig();
    } catch (error) {
      l("\n", error, "\n");
    }
  }

  async function cwQueryTokensWeight(addressList: string[] = []) {
    try {
      return await queryClient.queryTokensWeight({ addressList });
    } catch (error) {
      l("\n", error, "\n");
    }
  }

  async function cwQueryLiquidity(addressList: string[] = []) {
    try {
      return await queryClient.queryLiquidity({ addressList });
    } catch (error) {
      l("\n", error, "\n");
    }
  }

  async function cwQueryProviders(addressList: string[] = []) {
    try {
      return await queryClient.queryProviders({ addressList });
    } catch (error) {
      l("\n", error, "\n");
    }
  }

  async function cwQueryTokens(addressList: string[] = []) {
    try {
      return await queryClient.queryTokens({ addressList });
    } catch (error) {
      l("\n", error, "\n");
    }
  }

  async function cwQueryBalances(addressList: string[] = []) {
    try {
      return await queryClient.queryBalances({ addressList });
    } catch (error) {
      l("\n", error, "\n");
    }
  }

  async function cwQueryPrices(addressList: string[] = []) {
    try {
      return await queryClient.queryPrices({ addressList });
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
    const { execWrapper: _execWrapper } = await init(tokenAddr, seed);

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
    for (const [tokenAddr, symbol, priceFeedIdStr] of tokenInfoList) {
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
      } catch (error) {
        l("\n", error, "\n");
      }
    });

    await Promise.all(promiseList);

    balanceList = tokenAddrToSymbolList(balanceList);
    // l("\n", { Cw20Balances: balanceList }, "\n");

    return balanceList;
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
