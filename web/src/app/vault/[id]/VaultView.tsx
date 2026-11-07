"use client";
import { useCallback, useEffect, useSyncExternalStore } from "react";
import { ApiError, api, type Vault } from "@/lib/api";
import { PROGRAM_ID, displayName, shortAddress } from "@/lib/config";
import { formatDeadline, formatShortDeadline } from "@/lib/countdown";
import { OUTCOME_TEXT, TAG_OF, exitRules, funderTag, outcomeOf, shareOf, vaultFacts, type Tag } from "@/lib/outcome";
import { recentSignatures } from "@/lib/signatures";
import { formatUsdc } from "@/lib/usdc";
import { useWallet } from "@/components/wallet";
import { useAsync } from "@/components/useAsync";
import { ContributeForm } from "@/components/ContributeForm";
import { QuotePanel } from "@/components/QuotePanel";
import { DemoAttestor } from "@/components/DemoAttestor";
import { RecentTxs } from "@/components/RecentTxs";
import { NamedAddress, Problem, UpdatedAgo } from "@/components/parts";
import { AddressLink, Countdown, Loading, StatusBadge, Usdc } from "@/components/ui";
import { VaultMissing } from "./VaultMissing";

const REFUND_TEXT = {
  not_due: "Held until the visa decision or the deadline.",
  pending: "Refund pending: it is paid back to your wallet automatically.",
  refunded: "Refunded to your wallet.",
  unavailable: "This vault could not be found.",
};
const ROW_TAG: Record<ReturnType<typeof funderTag>, Tag> = {
  held: "open",
  "refund pending": "refunding",
  refunded: "refunded",
  "paid to the university": "released",
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
  const facts = vaultFacts(v, outcome, closedAt, fmtShort);
  const open = v.status === "open";

  return (
    <div className="vault">
      <div className="flex flex-wrap items-center gap-x-4 gap-y-2">
        <h1 className="page-title break-words">Deposit for {university}</h1>
        <StatusBadge outcome={outcome} />
      </div>
      {/* The site banner already says devnet and test money, so the intro does not repeat it. */}
      <p className="lead story mt-4">{OUTCOME_TEXT[outcome].body}</p>

      <div className="vault-grid">
        <div className="vault-summary">
          <p className="raised">
            {formatUsdc(v.total_contributed)} USDC<small>raised of {formatUsdc(v.target)} USDC</small>
          </p>
          <progress
            className={`bar bar-${tag} w-full`}
            max={v.target || 1}
            value={tag === "released" ? v.target || 1 : Math.min(v.total_contributed, v.target)}
            aria-label="Raised of target"
          />
          <dl className="facts">
            {facts.map((f) => (
              <div key={f.label}>
                <dt>{f.label}</dt>
                <dd>{"secondsLeft" in f ? <Countdown key={f.secondsLeft} secondsLeft={f.secondsLeft} /> : (f.text ?? "…")}</dd>
              </div>
            ))}
          </dl>
          <div className="mt-3 min-h-6"><UpdatedAgo at={vault.updatedAt} /></div>
        </div>

        <aside className="vault-panel" aria-labelledby="vault-panel-head">
          <h2 id="vault-panel-head">{open ? "Add to this deposit" : OUTCOME_TEXT[outcome].title}</h2>
          {/* Display only: the program enforces the deadline. */}
          {open && v.seconds_left > 0 ? (
            <>
              <ContributeForm vault={id} onDone={refresh} />
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
            <div className="mt-6">
              <h3 className="text-base font-semibold">Your contribution</h3>
              <Loading show={funder.loading && !funder.data} />
              <Problem error={funder.error} onRetry={reloadFunder} />
              {mine ? (
                <p className="text-sm"><Usdc micro={mine.amount} />: {REFUND_TEXT[mine.refund]}</p>
              ) : (
                funder.data && <p className="text-sm">You have not paid into this vault.</p>
              )}
            </div>
          )}
        </aside>

        <div className="vault-rest">
          {rules.length > 0 && (
            <div className={`rules${rules.length === 1 ? " rules-one" : ""}`}>
              {rules.map((r) => (
                <section key={r.key} className="card" aria-labelledby={`rule-${r.key}`}>
                  <h2 id={`rule-${r.key}`} className="section-head">{r.head}</h2>
                  <p className="text-sm">{r.text}</p>
                </section>
              ))}
            </div>
          )}

          <section className="funders" aria-labelledby="funders-head">
            <h2 id="funders-head">Who has paid</h2>
            {v.contributors.length === 0 ? (
              <p className="mt-4">Nobody has paid in yet.</p>
            ) : (
              <table>
                <thead>
                  <tr>
                    <th>Funder</th>
                    <th className="hide-sm">Wallet</th>
                    <th className="r">Paid</th>
                    <th className="r">Share</th>
                    <th className="hide-sm">Status</th>
                  </tr>
                </thead>
                <tbody>
                  {v.contributors.map((c) => {
                    const where = funderTag(outcome, c.refunded);
                    const name = displayName(c.funder);
                    const named = name !== shortAddress(c.funder);
                    const tag = <span className={`tag tag-${ROW_TAG[where]}`}>{where[0].toUpperCase() + where.slice(1)}</span>;
                    return (
                      <tr key={c.funder}>
                        {/* An unnamed funder is shown once, as the address link; on phones the status sits under it. */}
                        <td className={named ? "break-words" : "whitespace-nowrap"}>
                          {named ? name : <AddressLink address={c.funder} />}
                          <div className="row-tag">{tag}</div>
                        </td>
                        <td className="hide-sm">{named && <AddressLink address={c.funder} />}</td>
                        <td className="r">{formatUsdc(c.amount)}</td>
                        <td className="r">{shareOf(c.amount, v.total_contributed)}</td>
                        <td className="hide-sm">{tag}</td>
                      </tr>
                    );
                  })}
                </tbody>
              </table>
            )}
          </section>

          {open && (
            <details id="quote" className="quote-toggle mt-7">
              <summary>Paying from another chain? Get a quote through Aurora</summary>
              <div className="mt-4"><QuotePanel vault={id} /></div>
            </details>
          )}
          {health.data?.demo_attestor && open && (
            <div className="mt-7"><DemoAttestor vault={id} onDone={refresh} /></div>
          )}
          <div className="mt-7">
            <RecentTxs rows={txs.data} error={txs.error} onRetry={reloadTxs} />
          </div>
        </div>
      </div>

      <details className="vault-details">
        <summary>Details</summary>
        <dl className="listing text-sm">
          <div className="listing-row"><dt>Vault account</dt><dd><AddressLink address={v.vault} /></dd></div>
          <div className="listing-row"><dt>University</dt><dd><NamedAddress address={v.university} /></dd></div>
          <div className="listing-row"><dt>Visa attestor</dt><dd><NamedAddress address={v.attestor} /></dd></div>
          <div className="listing-row"><dt>Student</dt><dd><AddressLink address={v.student} /></dd></div>
          <div className="listing-row"><dt>Program</dt><dd><AddressLink address={PROGRAM_ID} /></dd></div>
          <div className="listing-row"><dt>Deadline</dt><dd>{fmt(v.deadline)}</dd></div>
        </dl>
      </details>
    </div>
  );
}
