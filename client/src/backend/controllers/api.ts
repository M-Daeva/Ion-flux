import { Request, Response } from "express";
import { transferTokens as _transferTokens } from "../middleware/api";

async function transferTokens(req: Request, res: Response) {
  const { recipient, tokenAddr } = req.body as unknown as {
    recipient: string | undefined;
    tokenAddr: string | undefined;
  };
  if (!recipient || !tokenAddr) return;

  const data = await _transferTokens(recipient, tokenAddr);
  res.send(data);
}

export { transferTokens };
