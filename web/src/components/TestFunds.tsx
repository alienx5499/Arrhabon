"use client";
// DEMO faucet: one click tops the signed-in wallet up with devnet SOL (fees) and test USDC (contributions).
import { useState } from "react";
import { api } from "@/lib/api";
import { faucetMessage } from "@/lib/faucet";

export function TestFunds({ address }: { address: string }) {
  const [busy, setBusy] = useState(false);
  const [message, setMessage] = useState<string | null>(null);
  async function claim() {
    setBusy(true);
    setMessage(null);
    try {
      setMessage(faucetMessage(await api.faucet(address)));
    } catch (e) {
      setMessage(faucetMessage(e));
    } finally {
      setBusy(false);
    }
  }
  return (
    <>
      <button className="btn-secondary" onClick={claim} disabled={busy} title="Devnet SOL for fees and test USDC to fund vaults; worth nothing">
        {busy ? "Sending…" : "Get test funds"}
      </button>
      <span role="status" aria-live="polite" className="text-sm">{message}</span>
    </>
  );
}
