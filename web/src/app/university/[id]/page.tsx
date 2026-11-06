"use client";
import { useCallback } from "react";
import { useParams } from "next/navigation";
import Link from "next/link";
import { api } from "@/lib/api";
import { useAsync } from "@/components/useAsync";
import { AddressLink, Card, ErrorText, Loading, StatusBadge, Usdc } from "@/components/ui";

export default function UniversityPage() {
  const { id } = useParams<{ id: string }>();
  const proofs = useAsync(useCallback(() => api.universityProof(id), [id]));

  return (
    <div className="flex flex-col gap-6">
      <h1 className="page-title">University</h1>
      <p className="story">
        Deposits held for <AddressLink address={id} />. Funds sit in a program-owned account and move only by the
        deployed program&apos;s rules: to you on release, or back to the funders on refund. The program&apos;s upgrade
        authority is disclosed in the README.
      </p>
      <Loading show={proofs.loading} />
      <ErrorText error={proofs.error} />
      {proofs.data?.length === 0 && <p className="text-sm">No vaults name this university.</p>}
      {proofs.data?.map((p) => (
        <Card key={p.vault} title={`Vault ${p.vault.slice(0, 4)}…${p.vault.slice(-4)}`}>
          <dl className="listing text-sm">
            <div className="listing-row"><dt>Status</dt><dd><StatusBadge status={p.status} /></dd></div>
            <div className="listing-row"><dt>Funds held</dt><dd><Usdc micro={p.funds_held} /> (target <Usdc micro={p.target} />)</dd></div>
            <div className="listing-row"><dt>Release condition</dt><dd>{p.release_condition}</dd></div>
            <div className="listing-row"><dt>Attestor</dt><dd><AddressLink address={p.attestor} /></dd></div>
            <div className="listing-row"><dt>Student</dt><dd><AddressLink address={p.student} /></dd></div>
            <div className="listing-row"><dt>Deadline</dt><dd>{new Date(p.deadline * 1000).toLocaleString()}</dd></div>
          </dl>
          <Link className="link mt-3 inline-block text-sm" href={`/vault/${p.vault}`}>Open vault page</Link>
        </Card>
      ))}
    </div>
  );
}
