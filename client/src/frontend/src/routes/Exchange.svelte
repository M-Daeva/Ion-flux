<script lang="ts">
  import { l } from "../../../common/utils";
  import { displayModal, getImageUrl } from "../services/helpers";
  import {
    contractCw20BalancesStorage,
    contractPricesStorage,
    initAll,
  } from "../services/storage";
  import { trimDecimal, symbolToAddr } from "../../../common/helpers/general";
  import { init } from "../workers/testnet-strat";

  let priceList: [string, string][] = [];
  let cw20Balances: [string, number][] = [];
  let symbolIn = "";
  let symbolOut = "";
  let amountIn = 0;

  $: priceIn = priceList.find(
    ([addr, price]) => addr === symbolToAddr(symbolIn)
  )?.[1];
  $: priceOut = priceList.find(
    ([addr, price]) => addr === symbolToAddr(symbolOut)
  )?.[1];
  $: exchangeRatio = +priceIn / +priceOut;
  $: amountOut = +trimDecimal(`${amountIn * exchangeRatio}`);

  contractCw20BalancesStorage.subscribe((value) => {
    cw20Balances = value;
    symbolIn = value?.[0]?.[0] || "";
    symbolOut = value?.[1]?.[0] || "";
  });

  contractPricesStorage.subscribe((value) => {
    priceList = value;
  });

  async function updatePrices() {
    const { cwQueryPrices } = await init();
    const prices = await cwQueryPrices();
    contractPricesStorage.set(prices);
  }

  async function swap() {
    try {
      const { cwSwap } = await init();
      const tx = await cwSwap(
        symbolToAddr(symbolIn),
        amountIn * 1e6,
        symbolToAddr(symbolOut)
      );
      l(tx);
      displayModal(tx);
      await initAll();
    } catch (error) {
      l(error);
    }
  }

  setInterval(updatePrices, 15_000);
</script>

<div class="flex flex-col px-4 -mt-3 pb-4">
  <div
    class="flex flex-row justify-around items-center mt-10 w-4/12 mx-auto py-5 sm:py-2 text-amber-200 font-medium my-2"
    style="background-color: rgb(42 48 60);"
  >
    <div class="flex flex-row py-5 justify-around w-9/12">
      <select
        id="symbol-selector"
        class="w-28 mx-0 bg-stone-700 my-auto"
        bind:value={symbolIn}
      >
        {#each cw20Balances as [tokenSymbol, _]}
          <option value={tokenSymbol}>
            {tokenSymbol}
          </option>
        {/each}
      </select>
      <input
        type="number"
        min="1"
        max="100"
        class="w-28 ml-2 my-auto text-center bg-stone-700"
        bind:value={amountIn}
      />
    </div>
  </div>

  <div class="flex justify-center">
    <img class="w-10" src={getImageUrl("arrow-down.png")} alt="arrow" />
  </div>

  <div
    class="flex flex-col justify-around items-center w-4/12 mx-auto py-5 sm:py-2 text-amber-200 font-medium my-2"
    style="background-color: rgb(42 48 60);"
  >
    <div class="flex flex-row py-5 justify-around w-9/12">
      <select
        id="symbol-selector"
        class="w-28 mx-0 bg-stone-700 my-auto"
        bind:value={symbolOut}
      >
        {#each cw20Balances as [tokenSymbol, _]}
          <option value={tokenSymbol}>
            {tokenSymbol}
          </option>
        {/each}
      </select>
      <input
        readonly
        type="number"
        min="1"
        max="100"
        class="w-28 ml-2 my-auto text-center bg-stone-700"
        bind:value={amountOut}
      />
    </div>
    <div class="flex flex-row justify-center">
      <span class="">1</span>
      <span class="ml-1">{symbolIn}</span>
      <span class="ml-1">=</span>
      <span class="ml-1">{trimDecimal(`${exchangeRatio}`)}</span>
      <span class="ml-1">{symbolOut}</span>
    </div>
    <button class="btn btn-secondary mt-5 w-28" on:click={swap}>Swap</button>
  </div>
</div>
