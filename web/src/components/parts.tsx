"use client";
// View pieces on top of ui.tsx: labelled cards, friendly errors with retry, freshness, named addresses.
import { useEffect, useId, useState, type ReactNode } from "react";
import { displayName, shortAddress } from "@/lib/config";
import { classifyProblem } from "@/lib/problem";
import { AddressLink } from "./ui";

const RETRY_MS = 5000;

/** A card whose heading names the section via aria-labelledby. */
export function Card({ title, children }: { title: ReactNode; children: ReactNode }) {
  const id = useId();
  return (
    <section className="card" aria-labelledby={id}>
      <h2 id={id} className="section-head">{title}</h2>
      {children}
    </section>
  );
}

/** Plain message; raw API/RPC text behind "Details". With onRetry, retries on its own when the error is temporary. */
export function Problem({ error, onRetry }: { error: unknown; onRetry?: () => void }) {
  const p = error ? classifyProblem(error) : null;
  const auto = Boolean(p?.retry && onRetry);
  useEffect(() => {
    if (!auto || !onRetry) return;
    const t = setTimeout(onRetry, RETRY_MS);
    return () => clearTimeout(t);
  }, [auto, onRetry, error]);
  if (!p) return null;
  return (
    <div role="alert" className="error flex flex-col items-start gap-2 text-sm">
      <p>
        {p.message}
        {auto && " Retrying…"}
      </p>
      {onRetry && (
        <button type="button" className="btn-secondary" onClick={onRetry}>
          Retry
        </button>
      )}
      {p.detail && (
        <details>
          <summary className="cursor-pointer">Details</summary>
          <code className="break-all">{p.detail}</code>
        </details>
      )}
    </div>
  );
}

/** "Updated 3s ago", ticking locally. */
export function UpdatedAgo({ at }: { at: number | null }) {
  const [now, setNow] = useState(() => Date.now());
  useEffect(() => {
    const t = setInterval(() => setNow(Date.now()), 1000);
    return () => clearInterval(t);
  }, []);
  if (at === null) return null;
  return <p className="muted text-sm">Updated {Math.max(0, Math.round((now - at) / 1000))}s ago</p>;
}

/** Registry display name with the explorer link; just the link when no name is known. */
export function NamedAddress({ address }: { address: string }) {
  const name = displayName(address);
  return (
    <span className="break-words">
      {name !== shortAddress(address) && <>{name} </>}
      <AddressLink address={address} />
    </span>
  );
}
