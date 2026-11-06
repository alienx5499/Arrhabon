"use client";
import { useState } from "react";
import { api } from "@/lib/api";
import { ErrorText, TxStatus } from "./ui";

/** Rendered only when the API reports the demo attestor enabled. */
export function DemoAttestor({ vault, onDone }: { vault: string; onDone: () => void }) {
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<unknown>(null);
  const [sig, setSig] = useState<string | null>(null);

  async function attest(outcome: "issued" | "denied") {
    setBusy(true);
    setError(null);
    try {
      setSig((await api.demoAttest(vault, outcome)).signature);
      onDone();
    } catch (e) {
      setError(e);
    } finally {
      setBusy(false);
    }
  }

  return (
    <section aria-label="DEMO attestor" className="demo-box">
      <h2 className="section-head mb-1">
        <span className="label demo-tag mr-2 align-middle">DEMO</span>
        Mock visa attestor
      </h2>
      <p className="mb-3 text-sm">Signs with a demo-only key. Not a real visa decision.</p>
      <div className="flex flex-wrap gap-3">
        <button className="btn" disabled={busy} onClick={() => attest("issued")}>DEMO: visa issued</button>
        <button className="btn-secondary" disabled={busy} onClick={() => attest("denied")}>DEMO: visa denied</button>
      </div>
      <ErrorText error={error} />
      <div className="mt-2"><TxStatus notice={null} sig={sig} /></div>
    </section>
  );
}
