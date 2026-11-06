"use client";
import { useCallback, useEffect } from "react";
import { useParams } from "next/navigation";
import { api } from "@/lib/api";
import { recentSignatures } from "@/lib/signatures";
import { useWallet } from "@/components/wallet";
import { useAsync } from "@/components/useAsync";
import { ContributeForm } from "@/components/ContributeForm";
import { QuotePanel } from "@/components/QuotePanel";
import { DemoAttestor } from "@/components/DemoAttestor";
import { RecentTxs } from "@/components/RecentTxs";
import { AddressLink, Card, Countdown, ErrorText, Loading, StatusBadge, Usdc } from "@/components/ui";

const REFUND_TEXT = {
  not_due: "Not due: held until the visa outcome or deadline",
  pending: "Refund pending: the crank or anyone can pay it out",
  refunded: "Refunded",
  unavailable: "Vault not found",
};
const POLL_MS = 5000;

export default function VaultPage() {
  const { id } = useParams<{ id: string }>();
  const { address } = useWallet();
  const vault = useAsync(useCallback(() => api.vault(id), [id]));
  const health = useAsync(api.health);
  const funderLoad = useCallback(() => api.funder(address!), [address]);
  const funder = useAsync(address ? funderLoad : null);
  const mine = funder.data?.contributions.find((c) => c.vault === id);
  const v = vault.data;
  const txs = useAsync(useCallback(() => recentSignatures(id), [id]));
  const refresh = () => (vault.reload(), funder.reload(), txs.reload());
  const live = v?.status === "open" || v?.status === "refunding";
  const { reload: reloadVault } = vault;
  const { reload: reloadFunder } = funder;
  const { reload: reloadTxs } = txs;

  // Poll while the vault can still change; once settled, one late read catches the closing transaction.
  useEffect(() => {
    if (!live) {
      const t = setTimeout(reloadTxs, POLL_MS);
      return () => clearTimeout(t);
    }
    const t = setInterval(() => (reloadVault(), reloadFunder(), reloadTxs()), POLL_MS);
    return () => clearInterval(t);
  }, [live, reloadVault, reloadFunder, reloadTxs]);

  return (
    <div className="flex flex-col gap-6">
      <div className="flex flex-wrap items-baseline gap-x-4 gap-y-2">
        <h1 className="page-title">Vault</h1>
        {v && <StatusBadge status={v.status} />}
      </div>
      <Loading show={vault.loading && !v} />
      <ErrorText error={vault.error} />
      {v && (
        <Card title="Status">
          <dl className="cols mb-4 border-b border-b-rule pb-4">
            <div className="fact">
              <dt className="label muted">Raised</dt>
              <dd className="fact-figure"><Usdc micro={v.total_contributed} /></dd>
            </div>
            <div className="fact">
              <dt className="label muted">Target</dt>
              <dd className="fact-figure"><Usdc micro={v.target} /></dd>
            </div>
            <div className="fact">
              <dt className="label muted">Time left</dt>
              <dd className="fact-figure"><Countdown key={v.seconds_left} secondsLeft={v.seconds_left} /></dd>
            </div>
          </dl>
          <progress className="mb-4 w-full" max={v.target || 1} value={Math.min(v.total_contributed, v.target)} aria-label="Raised of target" />
          <dl className="listing text-sm">
            <div className="listing-row"><dt>Status</dt><dd><StatusBadge status={v.status} /></dd></div>
            <div className="listing-row"><dt>Vault</dt><dd><AddressLink address={v.vault} /></dd></div>
            <div className="listing-row"><dt>Student</dt><dd><AddressLink address={v.student} /></dd></div>
            <div className="listing-row"><dt>University</dt><dd><AddressLink address={v.university} /></dd></div>
            <div className="listing-row"><dt>Raised</dt><dd><Usdc micro={v.total_contributed} /> of <Usdc micro={v.target} /></dd></div>
            <div className="listing-row">
              <dt>Deadline</dt>
              <dd>{new Date(v.deadline * 1000).toLocaleString()}</dd>
            </div>
          </dl>
          <button className="btn-secondary mt-3" onClick={refresh}>Refresh</button>
        </Card>
      )}
      {address && (
        <Card title="Your contribution">
          <Loading show={funder.loading && !funder.data} />
          <ErrorText error={funder.error} />
          {mine ? (
            <p className="text-sm"><Usdc micro={mine.amount} />: {REFUND_TEXT[mine.refund]}</p>
          ) : (
            funder.data && <p className="text-sm">You have not contributed to this vault.</p>
          )}
        </Card>
      )}
      {v?.status === "open" && (
        <div className="rule-section cols pt-3">
          {/* Display only: the program enforces the deadline. */}
          {v.seconds_left > 0 && <ContributeForm vault={id} onDone={refresh} />}
          <QuotePanel vault={id} />
        </div>
      )}
      {health.data?.demo_attestor && v?.status === "open" && <DemoAttestor vault={id} onDone={refresh} />}
      <div className="rule-section cols pt-3">
        {v && v.contributors.length > 0 && (
          <Card title="Funders">
            <ul className="listing text-sm">
              {v.contributors.map((c) => (
                <li key={c.funder} className="flex flex-wrap gap-3">
                  <AddressLink address={c.funder} /> <Usdc micro={c.amount} /> {c.refunded && <span className="label">refunded</span>}
                </li>
              ))}
            </ul>
          </Card>
        )}
        <RecentTxs rows={txs.data} error={txs.error} />
      </div>
    </div>
  );
}
