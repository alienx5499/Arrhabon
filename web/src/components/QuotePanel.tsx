"use client";
import { useState, type FormEvent } from "react";
import { api, type Quote } from "@/lib/api";
import { formatDeadline } from "@/lib/countdown";
import { CHAINS, percentToBps } from "@/lib/quote";
import { parseUsdc } from "@/lib/usdc";
import { useWallet } from "./wallet";
import { Card, Problem } from "./parts";
import { Field, Usdc } from "./ui";
import { LiveDeposit } from "./LiveDeposit";

/** Aurora cross-chain price check: dry quotes or live deposits. */
export function QuotePanel({ vault }: { vault: string }) {
  const wallet = useWallet();
  const [quote, setQuote] = useState<Quote | null>(null);
  const [origin, setOrigin] = useState<{ chain: string; asset: string; amount: string } | null>(null);
  const [error, setError] = useState<unknown>(null);
  const [busy, setBusy] = useState(false);

  async function submit(e: FormEvent<HTMLFormElement>) {
    e.preventDefault();
    const f = new FormData(e.currentTarget);
    const amount = parseUsdc(String(f.get("amount")));
    if (amount === null) return setError(new Error("Enter a USDC amount with at most 6 decimals."));
    const slippage = percentToBps(String(f.get("slippage")));
    if (slippage === null) return setError(new Error("Enter a price tolerance between 0 and 100 percent."));
    const chainId = String(f.get("chain"));
    const chainName = CHAINS.find((c) => c.id === chainId)?.name ?? chainId;
    const amountStr = String(f.get("amount"));
    setBusy(true);
    setError(null);
    try {
      const q = await api.quote({
        vault,
        origin_chain: chainId,
        origin_asset: "USDC",
        origin_amount: amount.toString(),
        funder: String(f.get("funder")),
        refund_to: String(f.get("refund_to")),
        slippage_bps: slippage,
      });
      setOrigin({ chain: chainName, asset: "USDC", amount: amountStr });
      setQuote(q);
    } catch (err) {
      setQuote(null);
      setOrigin(null);
      setError(err);
    } finally {
      setBusy(false);
    }
  }

  return (
    <Card title="Price check: nothing is sent">
      <p className="mb-3 text-sm muted">
        See how much USDC would reach this vault if you paid from another chain through Aurora. This only asks for a price; no
        money moves.
      </p>
      <form onSubmit={submit} className="grid grid-cols-1 gap-3 xl:grid-cols-2">
        <Field label="From chain">
          <select name="chain" className="input w-full max-w-full" defaultValue="base">
            {CHAINS.map((c) => <option key={c.id} value={c.id}>{c.name}</option>)}
          </select>
        </Field>
        <Field label="USDC to send">
          <input name="amount" required inputMode="decimal" className="input w-full max-w-full" placeholder="5" />
        </Field>
        <Field label="Your Solana address (receives USDC)">
          <input name="funder" required className="input w-full max-w-full font-mono" defaultValue={wallet.address ?? ""} key={wallet.address} />
        </Field>
        <Field label="Your address on that chain (for a refund if the swap fails)">
          <input name="refund_to" required className="input w-full max-w-full font-mono" placeholder="0x…" />
        </Field>
        <details className="min-w-0 text-sm xl:col-span-2">
          <summary className="cursor-pointer">Advanced</summary>
          <div className="mt-2">
            <Field label="Price tolerance (slippage, %)">
              <input name="slippage" inputMode="decimal" defaultValue="1" className="input w-full max-w-full" />
            </Field>
          </div>
        </details>
        <div className="flex items-end">
          <button className="btn-secondary" disabled={busy}>{busy ? "Checking…" : "Check price"}</button>
        </div>
      </form>
      <Problem error={error} />
      {quote && quote.dry && (
        <dl className="listing mt-4 text-sm" aria-live="polite">
          <div className="listing-row"><dt>At least this arrives</dt><dd><Usdc micro={quote.min_usdc_out} /></dd></div>
          <div className="listing-row"><dt>Fee</dt><dd><Usdc micro={quote.fee} /></dd></div>
          <div className="listing-row"><dt>Price valid until</dt><dd>{typeof quote.expires_at === "number" ? formatDeadline(quote.expires_at) : quote.expires_at}</dd></div>
          <div className="listing-row"><dt>Note</dt><dd>Price only: no deposit address was created.</dd></div>
        </dl>
      )}
      {quote && !quote.dry && origin && (
        <LiveDeposit quote={quote} chain={origin.chain} asset={origin.asset} amount={origin.amount} />
      )}
    </Card>
  );
}
