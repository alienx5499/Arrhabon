"use client";
import { useState, type FormEvent } from "react";
import { api } from "@/lib/api";
import { faucetMessage, isSolanaAddress } from "@/lib/faucet";
import { useWallet } from "@/components/wallet";
import { AddressLink, Card, Field, TxLink } from "@/components/ui";

export default function FaucetPage() {
  const { address: loggedInAddress } = useWallet();
  const [customAddress, setCustomAddress] = useState<string | null>(null);
  const address = customAddress ?? loggedInAddress ?? "";
  const [busy, setBusy] = useState(false);
  const [result, setResult] = useState<
    | { kind: "success"; message: string; sig: string; wallet: string }
    | { kind: "error"; message: string }
    | null
  >(null);

  async function submit(e: FormEvent<HTMLFormElement>) {
    e.preventDefault();
    const trimmed = address.trim();
    if (!isSolanaAddress(trimmed)) {
      setResult({ kind: "error", message: "That is not a Solana address." });
      return;
    }
    setBusy(true);
    setResult(null);
    try {
      const grant = await api.faucet(trimmed);
      setResult({
        kind: "success",
        message: faucetMessage(grant),
        sig: grant.signature,
        wallet: trimmed,
      });
    } catch (err) {
      setResult({
        kind: "error",
        message: faucetMessage(err),
      });
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className="flex flex-col gap-6">
      <h1 className="page-title">Test funds</h1>
      <p className="story">
        It sends 0.05 devnet SOL for fees and 1,000 test USDC to fund vaults, only what the wallet is missing, devnet only, worth nothing.
      </p>
      <Card title="Send test funds">
        <form onSubmit={submit} className="flex max-w-xl flex-col gap-4">
          <Field label="Wallet address">
            <input
              name="wallet"
              type="text"
              value={address}
              onChange={(e) => setCustomAddress(e.target.value)}
              required
              disabled={busy}
              className="input w-full max-w-full font-mono"
              placeholder="Solana address (base58)"
              autoComplete="off"
              spellCheck={false}
            />
          </Field>
          <button type="submit" className="btn self-start" disabled={busy}>
            {busy ? "Sending…" : "Send test funds"}
          </button>
          <div role="status" aria-live="polite" className="text-sm">
            {result?.kind === "success" && (
              <p className="flex flex-wrap items-baseline gap-2">
                <span>{result.message}</span>
                <TxLink sig={result.sig} />
                <AddressLink address={result.wallet} />
              </p>
            )}
            {result?.kind === "error" && (
              <p className="error">{result.message}</p>
            )}
          </div>
        </form>
      </Card>
      <p className="text-sm muted">
        Need more SOL?{" "}
        <a className="link" href="https://faucet.solana.com" target="_blank" rel="noreferrer">
          faucet.solana.com
        </a>
      </p>
    </div>
  );
}
