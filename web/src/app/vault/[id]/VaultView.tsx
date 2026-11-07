"use client";
import { useCallback, useEffect, useSyncExternalStore } from "react";
import { ApiError, api, type Vault } from "@/lib/api";
import { PROGRAM_ID, displayName } from "@/lib/config";
import { formatDeadline, formatShortDeadline } from "@/lib/countdown";
import { OUTCOME_TEXT, TAG_OF, exitRules, outcomeOf, vaultFacts } from "@/lib/outcome";
import { recentSignatures } from "@/lib/signatures";
import { formatUsdc } from "@/lib/usdc";
import { useWallet } from "@/components/wallet";
import { useAsync } from "@/components/useAsync";
import { ContributeForm } from "@/components/ContributeForm";
import { QuotePanel } from "@/components/QuotePanel";
import { DemoAttestor } from "@/components/DemoAttestor";
import { RecentTxs } from "@/components/RecentTxs";
import { NamedAddress, Problem, UpdatedAgo } from "@/components/parts";
import { AddressLink, Loading, Usdc } from "@/components/ui";
import { VaultMissing } from "./VaultMissing";
import { VaultBoard } from "./VaultBoard";
import { VaultExits } from "./VaultExits";
import { FundersTable } from "./FundersTable";
import "./vault.css";

const REFUND_TEXT = {
  not_due: "Held until the visa decision. If it is refused, or nobody answers by the deadline, it comes back to this wallet on its own.",
  pending: "Refund pending: it is paid back to your wallet automatically.",
  refunded: "Refunded to your wallet.",
  unavailable: "This vault could not be found.",
};
const POLL_MS = 5000;
const noSubscribe = () => () => {};
// The server renders times in UTC; the viewer's own zone takes over once hydrated, so the markup matches.
const serverTime = (unix: number) => formatDeadline(unix, "en-GB", "UTC");
const localTime = (unix: number) => formatDeadline(unix);
const serverShort = (unix: number) => formatShortDeadline(unix, "UTC");
const localShort = (unix: number) => formatShortDeadline(unix);

export function VaultView({ id, initial }: { id: string; initial: Vault | null }) {
  const { address } = useWallet();
  const hydrated = useSyncExternalStore(noSubscribe, () => true, () => false);
  const fmt = hydrated ? localTime : serverTime;
  const fmtShort = hydrated ? localShort : serverShort;
  const vault = useAsync(useCallback(() => api.vault(id), [id]), initial);
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

  // Poll while the vault can still change; once settled, one late read catches the closing transaction.
  useEffect(() => {
    if (!live) {
      const t = setTimeout(reloadTxs, POLL_MS);
      return () => clearTimeout(t);
    }
    const t = setInterval(refresh, POLL_MS);
    return () => clearInterval(t);
  }, [live, refresh, reloadTxs]);

  if (!v) {
    if (vault.error instanceof ApiError && vault.error.status === 404) return <VaultMissing />;
    return (
      // The min height holds room for the error box so the footer does not jump when loading turns into it.
      <div className="flex min-h-64 flex-col gap-6">
        <h1 className="page-title">Deposit vault</h1>
        <Loading show={vault.loading} />
        {/* Only the vault error shows while it fails; the Solana box waits for the vault. */}
        <Problem error={vault.error} onRetry={reloadVault} />
      </div>
    );
  }

  const outcome = outcomeOf(v.status, v.contributors);
  const tag = TAG_OF[outcome];
  const university = displayName(v.university);
  const rules = exitRules(v, outcome, university, formatUsdc);
  // Once closed, the newest transaction is the one that closed it; undefined while still looking.
  const closedAt = v.status !== "closed" ? null : txs.data ? (txs.data[0]?.blockTime ?? null) : txs.error ? null : undefined;
  const facts = [...vaultFacts(v, outcome, closedAt, fmtShort), { label: "Visa attestor", text: displayName(v.attestor) }];
  const open = v.status === "open";
  const toGo = open && v.total_contributed < v.target ? ` ${formatUsdc(v.target - v.total_contributed)} still to go before the university can be paid.` : "";

  return (
    <div className="vault">
      <VaultBoard title={`Deposit for ${university}`} outcome={outcome} facts={facts} />

      <div className="vault-main">
        <div className="vault-fig">
          <p className="vault-big">
            {formatUsdc(v.total_contributed)}
            <small>USDC</small>
          </p>
          <p className="vault-of">raised of {formatUsdc(v.target)} USDC.{toGo}</p>
          <progress
            className={`bar bar-${tag} w-full`}
            max={v.target || 1}
            value={tag === "released" ? v.target || 1 : Math.min(v.total_contributed, v.target)}
            aria-label="Raised of target"
          />
        </div>

        <aside className="vault-side" aria-labelledby="vault-side-head">
          <h2 id="vault-side-head">{open ? "Add to this deposit" : OUTCOME_TEXT[outcome].title}</h2>
          {/* Display only: the program enforces the deadline. */}
          {open && v.seconds_left > 0 ? (
            <>
              <ContributeForm
                vault={id}
                university={v.university}
                attestor={v.attestor}
                hint={`At least 1 USDC (or the exact remainder), up to the ${formatUsdc(Math.max(v.target - v.total_contributed, 0))} USDC still needed. You can pay until ${fmtShort(v.deadline)}.`}
                onDone={refresh}
              />
              <p className="note">
                Paying from another chain?{" "}
                <a className="link" href="#quote" onClick={() => document.getElementById("quote")?.setAttribute("open", "")}>
                  Get a quote through Aurora
                </a>
                .
              </p>
            </>
          ) : (
            <p className="text-sm">
              {open
                ? "The deadline has passed, so this vault no longer takes deposits. Refunds start automatically."
                : "This vault no longer takes deposits."}
            </p>
          )}
          {address && (
            <div className="mine">
              <Loading show={funder.loading && !funder.data} />
              <Problem error={funder.error} onRetry={reloadFunder} />
              {mine ? (
                <p><b>Your contribution: <Usdc micro={mine.amount} /></b>{REFUND_TEXT[mine.refund]}</p>
              ) : (
                funder.data && <p><b>Your contribution</b>You have not paid into this vault.</p>
              )}
            </div>
          )}
        </aside>

        <div className="vault-rest">
          {/* The site banner already says devnet and test money, so the story does not repeat it. */}
          <p className="vault-story">{OUTCOME_TEXT[outcome].body}</p>
          <div className="mt-2 min-h-6"><UpdatedAgo at={vault.updatedAt} /></div>
          <VaultExits rules={rules} />
        </div>
      </div>

      <FundersTable vault={v} outcome={outcome} me={address} />

      {open && (
        <details id="quote" className="quote-toggle mt-7">
          <summary>Paying from another chain? Get a quote through Aurora</summary>
          <div className="mt-4"><QuotePanel vault={id} /></div>
        </details>
      )}
      {health.data?.demo_attestor && open && (
        <div className="mt-7"><DemoAttestor vault={id} onDone={refresh} /></div>
      )}

      <div className="vault-lower">
        <RecentTxs rows={txs.data} error={txs.error} onRetry={reloadTxs} />
        <section className="vault-details" aria-labelledby="details-head">
          <h2 id="details-head">Details</h2>
          <dl className="listing text-sm">
            <div className="listing-row"><dt>Vault account</dt><dd><AddressLink address={v.vault} /></dd></div>
            <div className="listing-row"><dt>University</dt><dd><NamedAddress address={v.university} /></dd></div>
            <div className="listing-row"><dt>Visa attestor</dt><dd><NamedAddress address={v.attestor} /></dd></div>
            <div className="listing-row"><dt>Student</dt><dd><AddressLink address={v.student} /></dd></div>
            <div className="listing-row"><dt>Program</dt><dd><AddressLink address={PROGRAM_ID} /></dd></div>
            <div className="listing-row"><dt>Deadline</dt><dd>{fmt(v.deadline)}</dd></div>
          </dl>
        </section>
      </div>
    </div>
  );
}
