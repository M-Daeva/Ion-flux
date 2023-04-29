import { Decimal } from "decimal.js";
import TOKENS from "../config/tokens.json";

// [tokenAddr, symbol, priceFeedStr][]
const tokenInfoList: [string, string, string][] = [
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

function tokenAddrToSymbolList(addrAndValueList: [string, any][]) {
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

// removes additional digits on display
function trimDecimal(price: string | Decimal, err: string = "0.001"): string {
  price = price.toString();
  if (!price.includes(".")) return price;

  const one = new Decimal("1");
  const target = one.sub(new Decimal(err));

  let priceNext = price;
  let ratio = one;

  while (ratio.greaterThan(target)) {
    price = price.slice(0, price.length - 1);
    priceNext = price.slice(0, price.length - 1);
    ratio = new Decimal(priceNext).div(new Decimal(price));
  }

  return price.replace(/0/g, "") === "." ? "0" : price;
}

export { trimDecimal, tokenAddrToSymbolList, tokenInfoList };
