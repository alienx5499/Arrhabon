"use client";
import { useId, useState, type FormEvent } from "react";
import { api } from "@/lib/api";
import { parseUsdc } from "@/lib/usdc";
import { useWallet } from "./wallet";
import { useSignAndSubmit } from "./useSignAndSubmit";
import { Problem } from "./parts";
import { Field, TxStatus } from "./ui";

/** The pay form; the caller supplies the surrounding panel and heading. */
export function ContributeForm({ vault, onDone }: { vault: string; onDone: () => void }) {
  const wallet = useWallet();
  const tx = useSignAndSubmit();
  const [inputError, setInputError] = useState<string | null>(null);
  const why = useId();

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

  const disabledWhy = !wallet.address ? "Log in to pay from your wallet." : tx.busy ? "Approve the transaction in your wallet to continue." : null;
  return (
    <form onSubmit={submit} className="flex flex-col gap-3">
      <Field label="Amount in USDC">
        <input name="amount" required inputMode="decimal" className="input w-full max-w-full" placeholder="250" />
      </Field>
      <button className="btn-primary w-full" disabled={disabledWhy !== null} aria-describedby={disabledWhy ? why : undefined}>
        {tx.busy ? "Waiting for signature…" : "Sign and pay"}
      </button>
      {disabledWhy && <p id={why} className="text-sm">{disabledWhy}</p>}
      <Problem error={inputError ?? tx.error} />
      <TxStatus notice={tx.notice} sig={tx.signature} />
    </form>
  );
}
