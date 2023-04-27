import express from "express";
import { transferTokens } from "../controllers/api";

const router = express.Router();

router.post("/transfer-tokens", transferTokens);

export { router as api };
