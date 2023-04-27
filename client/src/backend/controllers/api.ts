import { Request, Response } from "express";
import { transferTokens as _transferTokens } from "../middleware/api";

async function transferTokens(req: Request, res: Response) {
  const { recipient } = req.body as unknown as {
    recipient: string | undefined;
  };
  if (!recipient) return;

  const data = await _transferTokens(recipient);
  res.send(data);
}

export { transferTokens };
