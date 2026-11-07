"use client";
import { useId, useState } from "react";
import { api } from "@/lib/api";
import { Problem } from "./parts";
import { TxStatus } from "./ui";

/** Rendered only when the API reports the demo attestor enabled. */
export function DemoAttestor({ vault, onDone }: { vault: string; onDone: () => void }) {
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<unknown>(null);
  const [sig, setSig] = useState<string | null>(null);
  const heading = useId();
  const why = useId();

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
    <section aria-labelledby={heading} className="demo-box">
      <h2 id={heading} className="section-head mb-1">
        <span className="label demo-tag mr-2 align-middle">Demo</span>
        Pretend visa decision
      </h2>
      <p className="mb-3 text-sm">For this demo only: decide the visa with a test key. A real vault waits for the real attestor.</p>
      <div className="flex flex-wrap gap-3">
        <button className="btn" disabled={busy} aria-describedby={busy ? why : undefined} onClick={() => attest("issued")}>Demo: visa issued</button>
        <button className="btn-secondary" disabled={busy} aria-describedby={busy ? why : undefined} onClick={() => attest("denied")}>Demo: visa denied</button>
      </div>
      {busy && <p id={why} className="mt-2 text-sm">Sending the decision…</p>}
      <Problem error={error} />
      <div className="mt-2"><TxStatus notice={null} sig={sig} /></div>
    </section>
  );
}
