"use client";
import { useCallback, useEffect } from "react";
import { api } from "@/lib/api";
import { displayName } from "@/lib/config";
import { formatDeadline } from "@/lib/countdown";
import { OUTCOME_TEXT, funderTag, outcomeOf } from "@/lib/outcome";
import { recentSignatures } from "@/lib/signatures";
import { useWallet } from "@/components/wallet";
import { useAsync } from "@/components/useAsync";
import { ContributeForm } from "@/components/ContributeForm";
import { QuotePanel } from "@/components/QuotePanel";
import { DemoAttestor } from "@/components/DemoAttestor";
import { RecentTxs } from "@/components/RecentTxs";
import { Card, NamedAddress, Problem, UpdatedAgo } from "@/components/parts";
import { AddressLink, Countdown, Loading, StatusBadge, Usdc } from "@/components/ui";

const REFUND_TEXT = {
  not_due: "Held until the visa decision or the deadline.",
  pending: "Refund pending: it is paid back to your wallet automatically.",
  refunded: "Refunded to your wallet.",
  unavailable: "This vault could not be found.",
};
const POLL_MS = 5000;

export function VaultView({ id }: { id: string }) {
  const { address, ready } = useWallet();
  const vault = useAsync(useCallback(() => api.vault(id), [id]));
  const health = useAsync(api.health);
  const funderLoad = useCallback(() => api.funder(address!), [address]);
  const funder = useAsync(address ? funderLoad : null);
  const mine = funder.data?.contributions.find((c) => c.vault === id);
  const v = vault.data;
  const txs = useAsync(useCallback(() => recentSignatures(id), [id]));
  const live = v?.status === "open" || v?.status === "refunding";
  const { reload: reloadVault } = vault;
  const { reload: reloadFunder } = funder;
  const { reload: reloadTxs } = txs;
  const refresh = useCallback(() => (reloadVault(), reloadFunder(), reloadTxs()), [reloadVault, reloadFunder, reloadTxs]);
  const outcome = v ? outcomeOf(v.status, v.contributors) : null;

  // Poll while the vault can still change; once settled, one late read catches the closing transaction.
  useEffect(() => {
    if (!live) {
      const t = setTimeout(reloadTxs, POLL_MS);
      return () => clearTimeout(t);
    }
    const t = setInterval(refresh, POLL_MS);
    return () => clearInterval(t);
  }, [live, refresh, reloadTxs]);

  return (
    <div className="flex flex-col gap-6">
      {ready && !address && (
        <Card title="What is this?">
          <p className="story text-sm">
            This is a tuition deposit vault. Family and friends add money towards a student&apos;s university deposit. A
            program on Solana holds it, not a person, and it can leave in only two ways: to the university if the visa is
            issued, or back to every funder if the visa is denied or the deadline passes.
          </p>
          <p className="mt-2 text-sm">This demo runs on Solana devnet with test money. No real funds move.</p>
        </Card>
      )}
      <div className="flex flex-wrap items-baseline gap-x-4 gap-y-2">
        <h1 className="page-title break-words">{v ? `Deposit for ${displayName(v.university)}` : "Deposit vault"}</h1>
        {v && <StatusBadge status={v.status} />}
      </div>
      <Loading show={vault.loading && !v} />
      <Problem error={vault.error} onRetry={reloadVault} />
      {v && outcome && (
        <Card title={OUTCOME_TEXT[outcome].title}>
          <p className="story mb-4 text-sm">{OUTCOME_TEXT[outcome].body}</p>
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
            <div className="listing-row"><dt>University</dt><dd><NamedAddress address={v.university} /></dd></div>
            <div className="listing-row"><dt>Visa attestor</dt><dd><NamedAddress address={v.attestor} /></dd></div>
            <div className="listing-row"><dt>Student</dt><dd><AddressLink address={v.student} /></dd></div>
            <div className="listing-row"><dt>Vault account</dt><dd><AddressLink address={v.vault} /></dd></div>
            <div className="listing-row"><dt>Deadline</dt><dd>{formatDeadline(v.deadline)}</dd></div>
          </dl>
          <div className="mt-3"><UpdatedAgo at={vault.updatedAt} /></div>
        </Card>
      )}
      {address && (
        <Card title="Your contribution">
          <Loading show={funder.loading && !funder.data} />
          <Problem error={funder.error} onRetry={reloadFunder} />
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
          {v.seconds_left > 0 ? (
            <ContributeForm vault={id} onDone={refresh} />
          ) : (
            <Card title="Contribute">
              <p className="text-sm">The deadline has passed, so this vault no longer takes deposits. Refunds start automatically.</p>
            </Card>
          )}
          <QuotePanel vault={id} />
        </div>
      )}
      {health.data?.demo_attestor && v?.status === "open" && <DemoAttestor vault={id} onDone={refresh} />}
      <div className="rule-section cols pt-3">
        {v && outcome && v.contributors.length > 0 && (
          <Card title="Funders">
            <ul className="listing text-sm">
              {v.contributors.map((c) => (
                <li key={c.funder} className="flex flex-wrap gap-3">
                  <AddressLink address={c.funder} /> <Usdc micro={c.amount} /> <span className="label">{funderTag(outcome, c.refunded)}</span>
                </li>
              ))}
            </ul>
          </Card>
        )}
        <RecentTxs rows={txs.data} error={txs.error} onRetry={reloadTxs} />
      </div>
    </div>
  );
}
