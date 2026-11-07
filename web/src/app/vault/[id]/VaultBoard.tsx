"use client";
// The navy board at the top of a vault page: title, status and the vault's facts in one frame.
import type { Fact, Outcome } from "@/lib/outcome";
import { Countdown, StatusBadge } from "@/components/ui";

export function VaultBoard({ title, outcome, facts }: { title: string; outcome: Outcome; facts: Fact[] }) {
  return (
    <section className="vboard" aria-label="This deposit">
      <div className="vboard-title">
        <h1 className="break-words">{title}</h1>
        <StatusBadge outcome={outcome} />
      </div>
      <dl className="vboard-facts">
        {facts.map((f) => (
          <div key={f.label}>
            <dt>{f.label}</dt>
            <dd>{"secondsLeft" in f ? <Countdown key={f.secondsLeft} secondsLeft={f.secondsLeft} /> : (f.text ?? "…")}</dd>
          </div>
        ))}
      </dl>
    </section>
  );
}
