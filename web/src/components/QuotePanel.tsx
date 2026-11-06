"use client";
import { useState, type FormEvent } from "react";
import { api, type Quote } from "@/lib/api";
import { parseUsdc } from "@/lib/usdc";
import { useWallet } from "./wallet";
import { Card, ErrorText, Field, Usdc } from "./ui";

const CHAINS = ["base", "arb", "eth"];

/** Aurora cross-chain price check: dry quotes only, nothing is sent. */
export function QuotePanel({ vault }: { vault: string }) {
  const wallet = useWallet();
  const [quote, setQuote] = useState<Quote | null>(null);
  const [error, setError] = useState<unknown>(null);
  const [busy, setBusy] = useState(false);

  async function submit(e: FormEvent<HTMLFormElement>) {
    e.preventDefault();
    const f = new FormData(e.currentTarget);
    const amount = parseUsdc(String(f.get("amount")));
    if (amount === null) return setError(new Error("Enter a USDC amount with at most 6 decimals."));
    setBusy(true);
    setError(null);
    try {
      setQuote(
        await api.quote({
          vault,
          origin_chain: String(f.get("chain")),
          origin_asset: "USDC",
          origin_amount: amount.toString(),
          funder: String(f.get("funder")),
          refund_to: String(f.get("refund_to")),
          slippage_bps: Number(f.get("slippage")),
        }),
      );
    } catch (err) {
      setQuote(null);
      setError(err);
    } finally {
      setBusy(false);
    }
  }

  return (
    <Card title="Fund from another chain (Aurora quote)">
      <form onSubmit={submit} className="grid gap-3 xl:grid-cols-2">
        <Field label="From chain">
          <select name="chain" className="input" defaultValue="base">
            {CHAINS.map((c) => <option key={c}>{c}</option>)}
          </select>
        </Field>
        <Field label="USDC to send">
          <input name="amount" required inputMode="decimal" className="input" placeholder="5" />
        </Field>
        <Field label="Your Solana address (receives USDC)">
          <input name="funder" required className="input font-mono" defaultValue={wallet.address ?? ""} key={wallet.address} />
        </Field>
        <Field label="Refund address on the origin chain">
          <input name="refund_to" required className="input font-mono" placeholder="0x…" />
        </Field>
        <Field label="Slippage (basis points)">
          <input name="slippage" type="number" min={0} max={10000} defaultValue={100} className="input" />
        </Field>
        <div className="flex items-end">
          <button className="btn" disabled={busy}>{busy ? "Quoting…" : "Get dry quote"}</button>
        </div>
      </form>
      <ErrorText error={error} />
      {quote && (
        <dl className="listing mt-4 text-sm" aria-live="polite">
          <div className="listing-row"><dt>Quote type</dt><dd>{quote.dry ? "Dry (price only, no deposit address)" : "Live"}</dd></div>
          <div className="listing-row"><dt>Guaranteed USDC out (minimum)</dt><dd><Usdc micro={quote.min_usdc_out} /></dd></div>
          <div className="listing-row"><dt>Fee</dt><dd><Usdc micro={quote.fee} /></dd></div>
          <div className="listing-row"><dt>Expires</dt><dd>{typeof quote.expires_at === "number" ? new Date(quote.expires_at * 1000).toLocaleString() : quote.expires_at}</dd></div>
        </dl>
      )}
    </Card>
  );
}
