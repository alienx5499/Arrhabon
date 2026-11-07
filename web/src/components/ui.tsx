"use client";
// Small presentational pieces shared by the views.
import { useEffect, useId, useState, type ReactNode } from "react";
import { TAG_OF, tagLabel, type Outcome } from "@/lib/outcome";
import { explorerAddress, explorerTx } from "@/lib/config";
import { formatTimeLeft } from "@/lib/countdown";
import { formatUsdc } from "@/lib/usdc";

export function StatusBadge({ outcome }: { outcome: Outcome }) {
  const tag = TAG_OF[outcome];
  return <span className={`tag tag-${tag}`}>{tagLabel(tag)}</span>;
}

export const Usdc = ({ micro }: { micro: number | bigint }) => (
  <span className="tabular-nums">{formatUsdc(micro)} USDC</span>
);

const short = (s: string) => (s.length > 12 ? `${s.slice(0, 4)}…${s.slice(-4)}` : s);

export const TxLink = ({ sig, label = "View transaction" }: { sig: string; label?: string }) => (
  <a className="link break-all" href={explorerTx(sig)} target="_blank" rel="noreferrer">
    {label} {short(sig)} (devnet explorer)
  </a>
);

/** Live region for signing progress and the confirmed transaction. */
export const TxStatus = ({ notice, sig }: { notice: string | null; sig: string | null }) => (
  <p role="status" className="text-sm">
    {notice}
    {sig && <>Confirmed: <TxLink sig={sig} /></>}
  </p>
);

export const AddressLink = ({ address }: { address: string }) => (
  <a className="link font-mono break-all" href={explorerAddress(address)} target="_blank" rel="noreferrer" title={address}>
    {short(address)}
  </a>
);

/** Ticks down locally from the API's seconds_left; the API stays the source of truth on refresh. */
export function Countdown({ secondsLeft }: { secondsLeft: number }) {
  const [start] = useState(() => Date.now());
  const [now, setNow] = useState(start);
  useEffect(() => {
    const t = setInterval(() => setNow(Date.now()), 1000);
    return () => clearInterval(t);
  }, []);
  return <span className="tabular-nums">{formatTimeLeft(secondsLeft - Math.floor((now - start) / 1000))}</span>;
}

export const Loading = ({ show }: { show: boolean }) =>
  show ? (
    <p role="status" aria-busy="true" className="text-sm">
      Loading…
    </p>
  ) : null;

export const ErrorText = ({ error }: { error: unknown }) =>
  error ? (
    <p role="alert" className="error text-sm">
      {error instanceof Error ? error.message : String(error)}
    </p>
  ) : null;

export function Card({ title, children }: { title: string; children: ReactNode }) {
  const id = useId();
  return (
    <section className="card" aria-labelledby={id}>
      <h2 id={id} className="section-head">{title}</h2>
      {children}
    </section>
  );
}

export function Field({ label, children }: { label: string; children: ReactNode }) {
  return (
    <label className="flex flex-col gap-1 text-sm">
      <span className="field-label">{label}</span>
      {children}
    </label>
  );
}
