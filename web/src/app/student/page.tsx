"use client";
import Link from "next/link";
import { useCallback } from "react";
import { api } from "@/lib/api";
import { useWallet } from "@/components/wallet";
import { useAsync } from "@/components/useAsync";
import { CreateVaultForm } from "@/components/CreateVaultForm";
import { Card, ErrorText, Loading, StatusBadge, Usdc } from "@/components/ui";

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
        <Loading show={vaults.loading} />
        <ErrorText error={vaults.error} />
        {vaults.data?.length === 0 && <p className="text-sm">No vaults yet.</p>}
        <ul className="listing">
          {vaults.data?.map((v) => (
            <li key={v.vault} className="flex flex-wrap items-baseline gap-3">
              <StatusBadge status={v.status} />
              <Link className="link font-mono text-sm break-all" href={`/vault/${v.vault}`}>{v.vault}</Link>
              <span className="text-sm">
                <Usdc micro={v.total_contributed} /> of <Usdc micro={v.target} />
              </span>
            </li>
          ))}
        </ul>
      </Card>
    </div>
  );
}
