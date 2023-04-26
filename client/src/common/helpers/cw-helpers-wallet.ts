import { l } from "../utils";
import { getCwClient, fee } from "../signers";
import { CONTRACT_ADDRESS } from "../config/testnet-config.json";
import { toUtf8 } from "@cosmjs/encoding";
import { MsgExecuteContract } from "cosmjs-types/cosmwasm/wasm/v1/tx";
import { UpdateConfigStruct, ClientStructWithKeplr } from "./interfaces";
import { IonFluxClient as Client } from "../codegen/IonFlux.client";
import { IonFluxMessageComposer as MessageComposer } from "../codegen/IonFlux.message-composer";
import { MsgExecuteContractEncodeObject } from "@cosmjs/cosmwasm-stargate";

const _toStr = (n?: number): string | undefined => (n ? `${n}` : undefined);

async function getCwHelpers(
  clientStruct: ClientStructWithKeplr,
  contractAddress: string
) {
  const cwClient = await getCwClient(clientStruct);
  if (!cwClient) return;

  const { client: _client, owner } = cwClient;
  const composer = new MessageComposer(owner, contractAddress);
  const client = new Client(_client, owner, contractAddress);

  async function _msgWrapper(msg: MsgExecuteContractEncodeObject) {
    const tx = await _client.signAndBroadcast(owner, [msg], fee);
    l("\n", tx, "\n");
    return tx;
  }

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
        sender: owner,
        contract: tokenAddr,
        msg: toUtf8(JSON.stringify(tokenMsg)),
        funds: undefined,
      }),
    };

    return await _msgWrapper(msgExecuteContractEncodeObject);
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
        sender: owner,
        contract: tokenAddr,
        msg: toUtf8(JSON.stringify(tokenMsg)),
        funds: undefined,
      }),
    };

    return await _msgWrapper(msgExecuteContractEncodeObject);
  }

  async function cwUpdateConfig(updateConfigStruct: UpdateConfigStruct) {
    const { admin, swapFeeRate, window, unbondingPeriod, priceAge } =
      updateConfigStruct;

    return await _msgWrapper(
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
    return await _msgWrapper(
      composer.updateToken({ tokenAddr, symbol, priceFeedIdStr })
    );
  }

  async function cwUnbond(tokenAddr: string, amount: number) {
    return await _msgWrapper(
      composer.unbond({ tokenAddr, amount: `${amount}` })
    );
  }

  async function cwWithdraw(tokenAddr: string, amount: number) {
    return await _msgWrapper(
      composer.withdraw({ tokenAddr, amount: `${amount}` })
    );
  }

  async function cwClaim() {
    return await _msgWrapper(composer.claim());
  }

  async function cwSwapAndClaim(tokenOutAddr: string) {
    return await _msgWrapper(composer.swapAndClaim({ tokenOutAddr }));
  }

  async function cwQueryConfig() {
    try {
      const res = await client.queryConfig();
      l("\n", res, "\n");
      return res;
    } catch (error) {
      l("\n", error, "\n");
    }
  }

  async function cwQueryTokensWeight(addressList: string[] = []) {
    try {
      const res = await client.queryTokensWeight({ addressList });
      l("\n", res, "\n");
      return res;
    } catch (error) {
      l("\n", error, "\n");
    }
  }

  async function cwQueryLiquidity(addressList: string[] = []) {
    try {
      const res = await client.queryLiquidity({ addressList });
      l("\n", res, "\n");
      return res;
    } catch (error) {
      l("\n", error, "\n");
    }
  }

  async function cwQueryProviders(addressList: string[] = []) {
    try {
      const res = await client.queryProviders({ addressList });
      l("\n", res, "\n");
      return res;
    } catch (error) {
      l("\n", error, "\n");
    }
  }

  async function cwQueryTokens(addressList: string[] = []) {
    try {
      const res = await client.queryTokens({ addressList });
      l("\n", res, "\n");
      return res;
    } catch (error) {
      l("\n", error, "\n");
    }
  }

  async function cwQueryBalances(addressList: string[] = []) {
    try {
      const res = await client.queryBalances({ addressList });
      l("\n", res, "\n");
      return res;
    } catch (error) {
      l("\n", error, "\n");
    }
  }

  async function cwQueryPrices(addressList: string[] = []) {
    try {
      const res = await client.queryPrices({ addressList });
      l("\n", res, "\n");
      return res;
    } catch (error) {
      l("\n", error, "\n");
    }
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
  };
}

export { getCwHelpers };
