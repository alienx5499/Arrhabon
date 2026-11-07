"use client";
import { useCallback } from "react";
import Link from "next/link";
import { api } from "@/lib/api";
import { displayName, shortAddress } from "@/lib/config";
import { formatDeadline } from "@/lib/countdown";
import { OUTCOME_TEXT, outcomeOf, universityTotals } from "@/lib/outcome";
import { useAsync } from "@/components/useAsync";
import { Card, NamedAddress, Problem } from "@/components/parts";
import { AddressLink, Loading, StatusBadge, Usdc } from "@/components/ui";

export function UniversityView({ id }: { id: string }) {
  const rows = useAsync(
    useCallback(async () => {
      const proofs = await api.universityProof(id);
      // shortcut: one vault read per proof, because the proof lacks contributor refunds; add totals to the proof API past a few dozen vaults.
      const vaults = await Promise.all(proofs.map((p) => api.vault(p.vault)));
      return proofs.map((proof, i) => ({ proof, vault: vaults[i] }));
    }, [id]),
  );
  const totals = rows.data && universityTotals(rows.data.map((r) => r.vault));
  // Until the first reply, hold the room the figures and cards will take; empty and error states keep it so nothing jumps.
  const pending = rows.loading && !rows.data && !rows.error;
  const blank = pending ? "…" : "—";
  const figure = (micro?: bigint) => (micro === undefined ? blank : <Usdc micro={micro} />);

  return (
    <div className="flex flex-col gap-6">
      <h1 className="page-title break-words">Deposits for {displayName(id)}</h1>
      <p className="story">
        Deposits held for <NamedAddress address={id} />. The money sits in accounts a Solana program controls and moves only by
        its rules: to you when the visa is issued, or back to the funders if it is denied or the deadline passes. Who can
        upgrade the program is disclosed in the README.
      </p>
      <dl className="cols">
          <div className="fact">
            <dt className="label muted">Held now</dt>
            <dd className="fact-figure">{figure(totals?.held)}</dd>
          </div>
          <div className="fact">
            <dt className="label muted">Paid to you</dt>
            <dd className="fact-figure">{figure(totals?.released)}</dd>
          </div>
          <div className="fact">
            <dt className="label muted">Refunded</dt>
            <dd className="fact-figure">{figure(totals?.refunded)}</dd>
          </div>
          <div className="fact">
            <dt className="label muted">Vaults</dt>
            <dd className="fact-figure tabular-nums">{totals ? totals.count : blank}</dd>
          </div>
      </dl>
      {/* At least the height of the two placeholder cards (h-72 each) and their gap. */}
      <div className="flex min-h-[37.5rem] flex-col gap-6">
      <Problem error={rows.error} onRetry={rows.reload} />
      {pending && (
        <>
          <div className="card h-72"><Loading show /></div>
          <div className="card h-72" aria-hidden="true" />
        </>
      )}
      {rows.data?.length === 0 && <p className="text-sm">No vaults name this university yet.</p>}
      {rows.data?.map(({ proof: p, vault: v }) => {
        const outcome = outcomeOf(p.status, v.contributors);
        return (
          <Card key={p.vault} title={`Student ${shortAddress(p.student)}`}>
            <dl className="listing text-sm">
              <div className="listing-row">
                <dt>Status</dt>
                <dd><StatusBadge outcome={outcome} /> {OUTCOME_TEXT[outcome].title}</dd>
              </div>
              <div className="listing-row"><dt>Deadline</dt><dd>{formatDeadline(p.deadline)}</dd></div>
              <div className="listing-row"><dt>Held now</dt><dd><Usdc micro={p.funds_held} /> (target <Usdc micro={p.target} />)</dd></div>
              <div className="listing-row">
                <dt>Paid to you when</dt>
                <dd>
                  <NamedAddress address={p.attestor} /> confirms the visa before the deadline with at least{" "}
                  <Usdc micro={p.target} /> held. Otherwise every funder is refunded.
                </dd>
              </div>
              <div className="listing-row"><dt>Student</dt><dd><AddressLink address={p.student} /></dd></div>
            </dl>
            <Link className="link mt-2 inline-block py-1 text-sm" href={`/vault/${p.vault}`}>Open this deposit</Link>
          </Card>
        );
      })}
      </div>
    </div>
  );
}
