<script lang="ts">
  import type { DeliverTxResponse } from "@cosmjs/cosmwasm-stargate";
  import { l, createRequest } from "../../../common/utils";
  import { displayModal } from "../services/helpers";
  import type { TxResponse } from "@injectivelabs/sdk-ts";
  import {
    contractCw20BalancesStorage,
    addressStorage,
  } from "../services/storage";
  import { get } from "svelte/store";
  import { baseURL } from "../config";
  import { init } from "../../../common/workers/testnet-frontend-workers";
  import { tokenInfoList } from "../../../common/helpers/general";

  let req = createRequest({ baseURL: baseURL + "/api" });

  let cw20Balances: [string, number][] = [];
  let currentSymbol = "";

  async function requestTokens() {
    try {
      const recipient = get(addressStorage);
      // sometimes it returns undefined while tx is ok
      const tx: DeliverTxResponse | TxResponse | undefined = await req.post(
        "/transfer-tokens",
        {
          recipient,
          tokenAddr: tokenInfoList.find(
            ([tokenAddr, symbol, priceFeedStr]) => symbol === currentSymbol
          )[0],
        }
      );

      displayModal(tx);

      const { cwQueryCw20Balances } = await init();
      cw20Balances = await cwQueryCw20Balances(recipient);
    } catch (error) {
      l(error);
    }
  }

  contractCw20BalancesStorage.subscribe((value) => {
    cw20Balances = value;
    currentSymbol = get(contractCw20BalancesStorage)?.[0]?.[0] || "";
  });
</script>

<div
  class="flex flex-col justify-center sm:flex-row sm:justify-between px-4 pb-4"
>
  <div class="w-full sm:w-4/12 flex justify-center items-center flex-col">
    <label for="symbol-selector" class="mr-2">Select Token</label>
    <select
      id="symbol-selector"
      class="w-28 m-0 bg-stone-700"
      bind:value={currentSymbol}
    >
      {#each get(contractCw20BalancesStorage) as [tokenSymbol, _]}
        <option value={tokenSymbol}>
          {tokenSymbol}
        </option>
      {/each}
    </select>

    <button class="btn btn-secondary mt-10 w-28" on:click={requestTokens}
      >Get Token</button
    >
  </div>

  <div class="mt-3 sm:mt-0 w-full sm:w-7/12 overflow-x-auto">
    <h2 class="text-center mb-3 text-lg">Balances</h2>

    <table class="table table-compact w-full overflow-x-scroll">
      <thead class="bg-black flex text-white w-full">
        <tr class="flex justify-between w-full mb-1 pr-6">
          <th class="bg-black py-4 w-24 text-center">Token</th>
          <th class="bg-black py-4 w-24 text-center">Amount</th>
        </tr>
      </thead>

      <tbody
        class="bg-grey-light flex flex-col items-center justify-start overflow-y-scroll w-full"
        style="max-height: 72vh; min-height: fit-content;"
      >
        {#each cw20Balances as cw20BalancesItem}
          <tr
            class="flex w-full mt-4 first:mt-0 justify-between pr-3"
            style="background-color: rgb(42 48 60);"
          >
            {#each cw20BalancesItem as rowValue}
              <td class="py-2.5 w-24 text-center bg-inherit border-b-0"
                >{rowValue}</td
              >
            {/each}
          </tr>
        {/each}
      </tbody>
    </table>
  </div>
</div>
