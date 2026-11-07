"use client";
import { useId, useState, type FormEvent } from "react";
import { api } from "@/lib/api";
import { formatUsdc, parseUsdc } from "@/lib/usdc";
import { useWallet } from "./wallet";
import { useSignAndSubmit } from "./useSignAndSubmit";
import { Problem } from "./parts";
import { TxStatus } from "./ui";

/** The pay form; the caller supplies the surrounding panel, heading and the hint under the amount. */
export function ContributeForm({ vault, hint, onDone }: { vault: string; hint: string; onDone: () => void }) {
  const wallet = useWallet();
  const tx = useSignAndSubmit();
  const [inputError, setInputError] = useState<string | null>(null);
  const [text, setText] = useState("");
  const ids = { field: useId(), hint: useId(), why: useId() };
  const typed = parseUsdc(text);

  async function submit(e: FormEvent<HTMLFormElement>) {
    e.preventDefault();
    const amount = parseUsdc(text);
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
    <form onSubmit={submit} className="pay-form">
      <label htmlFor={ids.field}>Amount</label>
      <div className="amount-field">
        <input id={ids.field} name="amount" required inputMode="decimal" placeholder="100.00" value={text} onChange={(e) => setText(e.target.value)} aria-describedby={ids.hint} />
        <span aria-hidden="true">USDC</span>
      </div>
      <p id={ids.hint} className="hint">{hint}</p>
      <button className="btn-primary pay" disabled={disabledWhy !== null} aria-describedby={disabledWhy ? ids.why : undefined}>
        {tx.busy ? "Waiting for signature…" : typed ? `Pay ${formatUsdc(typed)} USDC` : "Pay"}
      </button>
      {disabledWhy && <p id={ids.why} className="text-sm">{disabledWhy}</p>}
      <Problem error={inputError ?? tx.error} />
      <TxStatus notice={tx.notice} sig={tx.signature} />
    </form>
  );
}
