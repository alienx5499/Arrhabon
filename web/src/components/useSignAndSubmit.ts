"use client";
// API-built unsigned tx (base64) -> check it matches the intent -> user confirms -> wallet signs
// -> rebuild if the blockhash expired meanwhile -> POST /tx/submit -> signature.
import { useState } from "react";
import { createSolanaRpc } from "@solana/kit";
import { api, b64ToBytes, bytesToB64, type UnsignedTx } from "@/lib/api";
import { SOLANA_RPC_URL } from "@/lib/config";
import { checkTx, describeIntent, type Intent } from "@/lib/txCheck";
import { useWallet } from "./wallet";

const MAX_BUILDS = 3;
const rpc = createSolanaRpc(SOLANA_RPC_URL);

export function useSignAndSubmit() {
  const wallet = useWallet();
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<unknown>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const [signature, setSignature] = useState<string | null>(null);

  async function run<T extends UnsignedTx>(build: () => Promise<T>, intent: Intent, vaultOf?: (built: T) => string) {
    setBusy(true);
    setError(null);
    setNotice(null);
    setSignature(null);
    try {
      if (!wallet.address) throw new Error("Log in to sign");
      for (let attempt = 1; attempt <= MAX_BUILDS; attempt++) {
        const built = await build();
        const bytes = b64ToBytes(built.unsigned_tx);
        checkTx(bytes, wallet.address, intent);
        if (attempt === 1 && !window.confirm(describeIntent(intent, vaultOf?.(built)))) {
          throw new Error("Cancelled: nothing was signed.");
        }
        const signed = await wallet.sign(bytes);
        const height = await rpc.getBlockHeight().send();
        if (height > BigInt(built.last_valid_block_height)) {
          setNotice("Transaction expired, rebuilding…");
          continue;
        }
        const { signature } = await api.submit(bytesToB64(signed));
        setNotice(null);
        setSignature(signature);
        return { signature, built };
      }
      throw new Error("Transaction kept expiring before it could be submitted; try again.");
    } catch (e) {
      setNotice(null);
      setError(e);
      return null;
    } finally {
      setBusy(false);
    }
  }
  return { run, busy, error, notice, signature };
}
