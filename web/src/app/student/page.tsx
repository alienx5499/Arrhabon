"use client";
import Link from "next/link";
import { useCallback } from "react";
import { api } from "@/lib/api";
import { displayName } from "@/lib/config";
import { formatDeadline } from "@/lib/countdown";
import { OUTCOME_TEXT, outcomeOf } from "@/lib/outcome";
import { useWallet } from "@/components/wallet";
import { useAsync } from "@/components/useAsync";
import { CreateVaultForm } from "@/components/CreateVaultForm";
import { Card, Problem } from "@/components/parts";
import { Loading, StatusBadge, Usdc } from "@/components/ui";

export default function StudentPage() {
  const { address } = useWallet();
  const load = useCallback(() => api.studentVaults(address!), [address]);
  const vaults = useAsync(address ? load : null);

  return (
    <div className="flex flex-col gap-6">
      <h1 className="page-title">Student</h1>
      <CreateVaultForm onCreated={vaults.reload} />
      <Card title="Your vaults">
        {!address && <p className="text-sm">Log in to see your vaults.</p>}
        <Loading show={vaults.loading && !vaults.data} />
        <Problem error={vaults.error} onRetry={vaults.reload} />
        {vaults.data?.length === 0 && <p className="text-sm">No vaults yet.</p>}
        <ul className="listing">
          {vaults.data?.map((v) => (
            <li key={v.vault} className="flex flex-wrap items-baseline gap-3">
              <StatusBadge outcome={outcomeOf(v.status, v.contributors)} />
              <Link className="link break-words text-sm" href={`/vault/${v.vault}`}>Deposit for {displayName(v.university)}</Link>
              <span className="text-sm">
                {OUTCOME_TEXT[outcomeOf(v.status, v.contributors)].title}. <Usdc micro={v.total_contributed} /> of{" "}
                <Usdc micro={v.target} />, deadline {formatDeadline(v.deadline)}
              </span>
            </li>
          ))}
        </ul>
      </Card>
    </div>
  );
}
