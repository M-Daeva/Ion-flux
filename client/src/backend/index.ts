import express from "express";
import { l } from "../common/utils";
import { text, json } from "body-parser";
import cors from "cors";
import { rootPath } from "../common/utils";
import { api } from "./routes/api";
import { key } from "./routes/key";
import rateLimit from "express-rate-limit";
import * as h from "helmet";
import { PORT } from "./envs";

const limiter = rateLimit({
  windowMs: 60 * 1000, // 1 minute
  max: 30, // Limit each IP to 30 requests per `window`
  standardHeaders: true, // Return rate limit info in the `RateLimit-*` headers
  legacyHeaders: false, // Disable the `X-RateLimit-*` headers
  handler: (_req, res) => res.send("Request rate is limited"),
});

const staticHandler = express.static(rootPath("./dist/frontend"));

express()
  .disable("x-powered-by")
  .use(
    // h.contentSecurityPolicy(),
    h.crossOriginEmbedderPolicy({ policy: "credentialless" }),
    h.crossOriginOpenerPolicy(),
    h.crossOriginResourcePolicy(),
    h.dnsPrefetchControl(),
    h.expectCt(),
    h.frameguard(),
    h.hidePoweredBy(),
    h.hsts(),
    h.ieNoOpen(),
    h.noSniff(),
    // h.originAgentCluster(),
    h.permittedCrossDomainPolicies(),
    h.referrerPolicy(),
    h.xssFilter(),
    limiter,
    cors(),
    text(),
    json()
  )
  .use(staticHandler)
  .use("/api", api)
  .use("/key", key)
  .use("/*", staticHandler)
  .listen(PORT, async () => {
    l(`Ready on port ${PORT}`);
  });
