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

  return (
    <div className="flex flex-col gap-6">
      <h1 className="page-title break-words">Deposits for {displayName(id)}</h1>
      <p className="story">
        Deposits held for <NamedAddress address={id} />. The money sits in accounts a Solana program controls and moves only by
        its rules: to you when the visa is issued, or back to the funders if it is denied or the deadline passes. Who can
        upgrade the program is disclosed in the README.
      </p>
      <Loading show={rows.loading && !rows.data} />
      <Problem error={rows.error} onRetry={rows.reload} />
      {totals && (
        <dl className="cols border-b border-b-rule pb-4">
          <div className="fact">
            <dt className="label muted">Held now</dt>
            <dd className="fact-figure"><Usdc micro={totals.held} /></dd>
          </div>
          <div className="fact">
            <dt className="label muted">Paid to you</dt>
            <dd className="fact-figure"><Usdc micro={totals.released} /></dd>
          </div>
          <div className="fact">
            <dt className="label muted">Refunded</dt>
            <dd className="fact-figure"><Usdc micro={totals.refunded} /></dd>
          </div>
          <div className="fact">
            <dt className="label muted">Vaults</dt>
            <dd className="fact-figure tabular-nums">{totals.count}</dd>
          </div>
        </dl>
      )}
      {rows.data?.length === 0 && <p className="text-sm">No vaults name this university yet.</p>}
      {rows.data?.map(({ proof: p, vault: v }) => {
        const outcome = outcomeOf(p.status, v.contributors);
        return (
          <Card key={p.vault} title={`Student ${shortAddress(p.student)}, deadline ${formatDeadline(p.deadline)}`}>
            <dl className="listing text-sm">
              <div className="listing-row">
                <dt>Status</dt>
                <dd><StatusBadge status={p.status} /> {OUTCOME_TEXT[outcome].title}</dd>
              </div>
              <div className="listing-row"><dt>Held now</dt><dd><Usdc micro={p.funds_held} /> (target <Usdc micro={p.target} />)</dd></div>
              <div className="listing-row">
                <dt>Paid to you when</dt>
                <dd>
                  <NamedAddress address={p.attestor} /> confirms the visa before {formatDeadline(p.deadline)} with at least{" "}
                  <Usdc micro={p.target} /> held. Otherwise every funder is refunded.
                </dd>
              </div>
              <div className="listing-row"><dt>Student</dt><dd><AddressLink address={p.student} /></dd></div>
            </dl>
            <Link className="link mt-3 inline-block text-sm" href={`/vault/${p.vault}`}>Open this deposit</Link>
          </Card>
        );
      })}
    </div>
  );
}
