import { l } from "../../../common/utils";
import { tokenAddrToSymbolList } from "../../../common/helpers/general";
import { initWithKeplr } from "./injective-strat";
import { getCwClient } from "../../../common/signers";
import { CONTRACT_ADDRESS } from "../../../common/config/testnet-config.json";
import { toUtf8 } from "@cosmjs/encoding";
import { MsgExecuteContract } from "cosmjs-types/cosmwasm/wasm/v1/tx";
import type {
  UpdateConfigStruct,
  ClientStructWithKeplr,
} from "../../../common/helpers/interfaces";
import { IonFluxClient as Client } from "../../../common/codegen/IonFlux.client";
import { IonFluxMessageComposer as MessageComposer } from "../../../common/codegen/IonFlux.message-composer";
import type { MsgExecuteContractEncodeObject } from "@cosmjs/cosmwasm-stargate";
import TOKENS from "../../../common/config/tokens.json";
import { WalletStrategy } from "@injectivelabs/wallet-ts";
import { toBase64 } from "@injectivelabs/sdk-ts";

const _toStr = (n?: number): string | undefined => (n ? `${n}` : undefined);

async function getCwHelpers(
  walletStrategy: WalletStrategy,
  clientStruct: ClientStructWithKeplr
) {
  const { injectiveAddress: owner, execWrapper } = await initWithKeplr(
    CONTRACT_ADDRESS,
    walletStrategy
  );

  const composer = new MessageComposer(owner, CONTRACT_ADDRESS);

  const cwClient = await getCwClient(clientStruct);
  if (!cwClient) return;

  const { client: _client } = cwClient;
  const client = new Client(_client, owner, CONTRACT_ADDRESS);

  async function cwDeposit(tokenAddr: string, amount: number) {
    const { injectiveAddress: owner, execWrapper: _execWrapper } =
      await initWithKeplr(tokenAddr, walletStrategy);

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
        sender: owner,
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
    const { injectiveAddress: owner, execWrapper: _execWrapper } =
      await initWithKeplr(tokenAddr, walletStrategy);

    const contractMsg = {
      swap: {
        token_out_addr: tokenOutAddr,
      },
    };

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
        sender: owner,
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
      return await client.queryConfig();
    } catch (error) {
      l("\n", error, "\n");
    }
  }

  async function cwQueryTokensWeight(addressList: string[] = []) {
    try {
      return await client.queryTokensWeight({ addressList });
    } catch (error) {
      l("\n", error, "\n");
    }
  }

  async function cwQueryLiquidity(addressList: string[] = []) {
    try {
      return await client.queryLiquidity({ addressList });
    } catch (error) {
      l("\n", error, "\n");
    }
  }

  async function cwQueryProviders(addressList: string[] = []) {
    try {
      return await client.queryProviders({ addressList });
    } catch (error) {
      l("\n", error, "\n");
    }
  }

  async function cwQueryTokens(addressList: string[] = []) {
    try {
      return await client.queryTokens({ addressList });
    } catch (error) {
      l("\n", error, "\n");
    }
  }

  async function cwQueryBalances(addressList: string[] = []) {
    try {
      return await client.queryBalances({ addressList });
    } catch (error) {
      l("\n", error, "\n");
    }
  }

  async function cwQueryPrices(addressList: string[] = []) {
    try {
      return await client.queryPrices({ addressList });
    } catch (error) {
      l("\n", error, "\n");
    }
  }

  async function cwQueryCw20Balances(wallet: string) {
    const tokens: [string, string][] = Object.entries(TOKENS);
    let balanceList: [string, number][] = [];

    const promiseList = tokens.map(async ([k, v]) => {
      if (k === "CONTRACT_CODE") return;

      try {
        const res: { balance: string } = await client.client.queryContractSmart(
          v,
          {
            balance: { address: wallet },
          }
        );

        balanceList.push([v, +res.balance / 1e6]);
      } catch (error) {
        l("\n", error, "\n");
      }
    });

    await Promise.all(promiseList);

    balanceList = tokenAddrToSymbolList(balanceList);
    //l("\n", { Cw20Balances: balanceList }, "\n");

    return balanceList;
  }

  return {
    owner,

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

    cwQueryCw20Balances,
  };
}

export { getCwHelpers };
