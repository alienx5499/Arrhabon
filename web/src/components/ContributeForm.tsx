"use client";
import { useState, type FormEvent } from "react";
import { api } from "@/lib/api";
import { parseUsdc } from "@/lib/usdc";
import { useWallet } from "./wallet";
import { useSignAndSubmit } from "./useSignAndSubmit";
import { Card, ErrorText, Field, TxStatus } from "./ui";

export function ContributeForm({ vault, onDone }: { vault: string; onDone: () => void }) {
  const wallet = useWallet();
  const tx = useSignAndSubmit();
  const [inputError, setInputError] = useState<string | null>(null);

  async function submit(e: FormEvent<HTMLFormElement>) {
    e.preventDefault();
    const amount = parseUsdc(String(new FormData(e.currentTarget).get("amount")));
    if (amount === null) return setInputError("Enter a USDC amount with at most 6 decimals.");
    setInputError(null);
    const done = await tx.run(() => api.contribute(vault, { funder: wallet.address!, amount: Number(amount) }), {
      action: "contribute",
      vault,
      amount,
    });
    if (done) onDone();
  }

  return (
    <Card title="Contribute (devnet USDC)">
      <form onSubmit={submit} className="flex flex-col gap-3">
        <Field label="Amount (USDC)">
          <input name="amount" required inputMode="decimal" className="input" placeholder="250" />
        </Field>
        <button className="btn self-start" disabled={!wallet.address || tx.busy}>
          {tx.busy ? "Waiting for signature…" : "Sign and contribute"}
        </button>
        {!wallet.address && <p className="text-sm">Log in to contribute from your wallet.</p>}
        <ErrorText error={inputError ?? tx.error} />
        <TxStatus notice={tx.notice} sig={tx.signature} />
      </form>
    </Card>
  );
}
