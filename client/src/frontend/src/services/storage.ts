import { type Writable, writable } from "svelte/store";
import { l } from "../../../common/utils";
import { init } from "../../../common/workers/testnet-frontend-workers";
import type { NetworkData } from "../../../common/helpers/interfaces";
import type {
  ArrayOfTupleOfAddrAndArrayOfAsset,
  ArrayOfTupleOfAddrAndDecimal,
  ArrayOfTupleOfAddrAndToken,
  ArrayOfTupleOfAddrAndUint128,
  Config,
} from "../../../common/codegen/IonFlux.types";

// global constants
const LOCAL_STORAGE_KEY = "ion-flux-inj-addr";
const CHAIN_TYPE: "main" | "test" = "test";

// contract storages
let contractConfigStorage: Writable<Config> = writable();
let contractCw20BalancesStorage: Writable<[string, number][]> = writable([]);
let contractLiquidityStorage: Writable<ArrayOfTupleOfAddrAndUint128> = writable(
  []
);
let contractPricesStorage: Writable<ArrayOfTupleOfAddrAndDecimal> = writable(
  []
);
let contractProvidersStorage: Writable<ArrayOfTupleOfAddrAndArrayOfAsset> =
  writable([]);
let contractTokensStorage: Writable<ArrayOfTupleOfAddrAndToken> = writable([]);
let contractTokensWeightStorage: Writable<ArrayOfTupleOfAddrAndDecimal> =
  writable([]);

// frontend storages
let addressStorage: Writable<string> = writable();
// controls tx hash modal
let isModalActiveStorage: Writable<boolean> = writable(false);
// keeps last tx hash
let txResStorage: Writable<["Success" | "Error", string]> = writable([
  "Success",
  "",
]);

async function _initStorage(
  storage: Writable<any>,
  querier: () => Promise<any>
) {
  const res = await querier();
  storage.set(res);
}

async function initAll() {
  try {
    const address = localStorage.getItem(LOCAL_STORAGE_KEY);
    addressStorage.set(address);

    const {
      cwQueryConfig,
      cwQueryCw20Balances,
      cwQueryLiquidity,
      cwQueryPrices,
      cwQueryProviders,
      cwQueryTokens,
      cwQueryTokensWeight,
    } = await init();

    await Promise.all(
      [
        [contractConfigStorage, cwQueryConfig],
        [contractLiquidityStorage, cwQueryLiquidity],
        [contractPricesStorage, cwQueryPrices],
        [contractProvidersStorage, cwQueryProviders],
        [contractTokensStorage, cwQueryTokens],
        [contractTokensWeightStorage, cwQueryTokensWeight],
        [
          contractCw20BalancesStorage,
          async () => await cwQueryCw20Balances(address),
        ],
      ].map(([s, q]) =>
        _initStorage(
          s as unknown as Writable<any>,
          q as unknown as () => Promise<any>
        )
      )
    );
  } catch (error) {
    l(error);
  }
}

export {
  LOCAL_STORAGE_KEY,
  CHAIN_TYPE,
  addressStorage,
  isModalActiveStorage,
  txResStorage,
  initAll,
  contractConfigStorage,
  contractCw20BalancesStorage,
  contractLiquidityStorage,
  contractPricesStorage,
  contractProvidersStorage,
  contractTokensStorage,
  contractTokensWeightStorage,
};
