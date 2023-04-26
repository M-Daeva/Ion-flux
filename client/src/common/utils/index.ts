import axios, { AxiosRequestConfig } from "axios";
import path from "path";
import { SHA256, AES, enc } from "crypto-js";
import { TimeInHoursAndMins } from "../helpers/interfaces";
import { BigNumberInBase, BigNumber } from "@injectivelabs/utils";

const l = console.log.bind(console);

function r(num: number, digits: number = 0): number {
  let k = 10 ** digits;
  return Math.round(k * num) / k;
}

function getLast<T>(arr: T[]) {
  return arr[arr.length - 1];
}

function rootPath(dir: string) {
  return path.resolve(__dirname, "../../../", dir);
}

const SEP =
  "////////////////////////////////////////////////////////////////////////////////////\n";

function createRequest(config: Object) {
  const req = axios.create(config);

  return {
    get: async (url: string, config?: Object) => {
      return (await req.get(url, config)).data;
    },
    post: async (url: string, params: Object, config?: AxiosRequestConfig) => {
      return (await req.post(url, params, config)).data;
    },
    put: async (url: string, params: Object, config?: AxiosRequestConfig) => {
      return (await req.put(url, params, config)).data;
    },
    patch: async (url: string, params: Object, config?: AxiosRequestConfig) => {
      return (await req.patch(url, params, config)).data;
    },
  };
}

async function specifyTimeout(
  promise: Promise<any>,
  timeout: number = 5_000,
  exception: Function = () => {
    throw new Error("Timeout!");
  }
) {
  let timer: NodeJS.Timeout;

  return Promise.race([
    promise,
    new Promise((_r, rej) => (timer = setTimeout(rej, timeout, exception))),
  ]).finally(() => clearTimeout(timer));
}

function encrypt(data: string, key: string): string {
  return AES.encrypt(data, key).toString();
}

function decrypt(encryptedData: string, key: string): string | undefined {
  // "Malformed UTF-8 data" workaround
  try {
    const bytes = AES.decrypt(encryptedData, key);
    return bytes.toString(enc.Utf8);
  } catch (error) {
    return;
  }
}

function _timeToMins(time: TimeInHoursAndMins): number {
  const { hours, minutes } = time;
  return 60 * hours + minutes;
}

function _minsToTime(mins: number): TimeInHoursAndMins {
  const hours = Math.floor(mins / 60);
  const minutes = mins % 60;
  return { hours, minutes };
}

function calcTimeDelta(
  targetTime: TimeInHoursAndMins,
  period: TimeInHoursAndMins,
  ignoreRange: [TimeInHoursAndMins, TimeInHoursAndMins] | [] = []
): TimeInHoursAndMins {
  const targetTimeInMins = _timeToMins(targetTime);
  const currentTime = new Date();
  const currentTimeInMins = _timeToMins({
    hours: currentTime.getHours(),
    minutes: currentTime.getMinutes(),
  });
  const periodInMins = _timeToMins(period);

  let delta = currentTimeInMins - targetTimeInMins;
  if (delta < 0) delta += 24 * 60;

  let res = Math.ceil(delta / periodInMins) * periodInMins - delta;

  if (ignoreRange.length) {
    const [ignoreStartInMins, ignoreEndInMins] = ignoreRange.map(_timeToMins);

    if (
      currentTimeInMins + res >= ignoreStartInMins &&
      currentTimeInMins + res <= ignoreEndInMins
    ) {
      while (currentTimeInMins + res <= ignoreEndInMins) {
        res += periodInMins;
      }
    }
  }

  return _minsToTime(res);
}

function fromMicroToDecimal(micro: string): number {
  return new BigNumber(micro)
    .div(new BigNumber(10).exponentiatedBy(18))
    .toNumber();
}

function fromDecimalToMicro(decimal: number): string {
  return new BigNumberInBase(decimal).toWei().toFixed();
}

export {
  l,
  r,
  createRequest,
  rootPath,
  SEP,
  getLast,
  specifyTimeout,
  encrypt,
  decrypt,
  calcTimeDelta,
  fromDecimalToMicro,
  fromMicroToDecimal,
};
