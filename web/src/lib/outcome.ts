// Plain-language reading of a vault's state; pure, so the views and tests share one source.
import type { Vault, VaultStatus } from "./api";

type Contributor = Vault["contributors"][number];
/** "closed" splits by how it ended: paid to the university, refunded to everyone, or never funded. */
export type Outcome = "open" | "released" | "paid" | "refunding" | "refunded" | "empty";

export function outcomeOf(status: VaultStatus, contributors: Contributor[]): Outcome {
  if (status !== "closed") return status;
  if (contributors.length === 0) return "empty";
  // A vault closes after refunds only once every funder is refunded; after release nobody is.
  return contributors.every((c) => c.refunded) ? "refunded" : "paid";
}

export const OUTCOME_TEXT: Record<Outcome, { title: string; body: string }> = {
  open: {
    title: "Collecting deposits",
    body: "Family and friends pay towards this student's university deposit. A Solana program holds it, not a person, and it can only go to the university or back to the people who paid.",
  },
  released: {
    title: "Visa issued: paying the university",
    body: "The visa was confirmed. The deposit is being paid to the university; nobody needs to do anything.",
  },
  paid: {
    title: "Closed: paid to the university",
    body: "The visa was issued and the university has received the deposit. Nothing more will happen to this vault.",
  },
  refunding: {
    title: "Refunding every funder",
    body: "The visa was denied or the deadline passed. Every funder is being refunded automatically; nobody can redirect this money.",
  },
  refunded: {
    title: "Closed: every funder refunded",
    body: "The visa was denied or the deadline passed, and every funder has been paid back. Nothing more will happen to this vault.",
  },
  empty: {
    title: "Closed with no deposits",
    body: "The deadline passed before anyone added money. Nothing was held and nothing more will happen.",
  },
};

/** Where one funder's money is now. */
export function funderTag(outcome: Outcome, refunded: boolean): string {
  if (refunded) return "refunded";
  if (outcome === "refunding" || outcome === "refunded") return "refund pending";
  if (outcome === "released" || outcome === "paid") return "paid to the university";
  return "held";
}

/** Totals across a university's vaults, in USDC micro-units. */
export function universityTotals(vaults: Vault[]) {
  let held = 0n;
  let released = 0n;
  let refunded = 0n;
  for (const v of vaults) {
    const back = v.contributors.reduce((sum, c) => (c.refunded ? sum + BigInt(c.amount) : sum), 0n);
    const outcome = outcomeOf(v.status, v.contributors);
    refunded += back;
    if (outcome === "released" || outcome === "paid") released += BigInt(v.total_contributed);
    else if (outcome === "open" || outcome === "refunding") held += BigInt(v.total_contributed) - back;
  }
  return { held, released, refunded, count: vaults.length };
}

/** The status tags: a closed vault reads "Released" when paid out, "Refunded" when paid back, "Closed" when never funded. */
export type Tag = "open" | "released" | "refunding" | "refunded" | "closed";
export const TAG_OF: Record<Outcome, Tag> = {
  open: "open",
  released: "released",
  paid: "released",
  refunding: "refunding",
  refunded: "refunded",
  empty: "closed",
};
export const tagLabel = (t: Tag) => t[0].toUpperCase() + t.slice(1);

/** One funder's share of the total as a whole percent; "<1%" rather than a misleading "0%". */
export function shareOf(amount: number, total: number): string {
  if (total <= 0 || amount <= 0) return "0%";
  const pct = Math.round((amount * 100) / total);
  return pct === 0 ? "<1%" : `${pct}%`;
}

/** A fact under the big figure; `secondsLeft` renders as a live countdown, a null `text` as a placeholder. */
export type Fact = { label: string; text: string | null } | { label: string; secondsLeft: number };

/**
 * Facts for the vault page. Only an open vault has time left; a closed one shows when it closed
 * (`closedAt` undefined = still looking it up, null = not derivable). The slot stays either way so the row never jumps.
 */
export function vaultFacts(v: Vault, outcome: Outcome, closedAt: number | null | undefined, fmt: (unix: number) => string): Fact[] {
  const facts: Fact[] = [];
  if (outcome === "open") facts.push({ label: "Time left", secondsLeft: v.seconds_left });
  if (v.status === "closed") facts.push({ label: "Closed on", text: closedAt === undefined ? null : closedAt === null ? "Not known" : fmt(closedAt) });
  facts.push({ label: "Deadline", text: fmt(v.deadline) }, { label: "Funders", text: String(v.contributors.length) });
  return facts;
}

export type ExitRule = { key: "issued" | "refused"; head: string; text: string };

/**
 * The exit cards written with this vault's numbers; `usdc` formats micro-units without the unit.
 * An open vault shows both exits, a finished one only the exit it took, a closed empty one none.
 */
export function exitRules(v: Vault, outcome: Outcome, university: string, usdc: (micro: number) => string): ExitRule[] {
  const n = v.contributors.length;
  if (outcome === "released" || outcome === "paid") {
    return [{ key: "issued", head: "Paid to the university", text: `The visa was issued. The whole deposit went to ${university}.` }];
  }
  if (outcome === "refunded") {
    const who = n === 1 ? "The funder got" : `Each of the ${n} funders got`;
    const text = `The visa was refused or the deadline passed. ${who} back their own amount, ${usdc(v.total_contributed)} USDC in all.`;
    return [{ key: "refused", head: "Every funder was refunded", text }];
  }
  if (outcome === "empty") return [];
  const { issued, refused } = openExits(v, university, usdc);
  const head = outcome === "refunding" ? "Every funder is being refunded" : "If it is refused or the deadline passes";
  const refusedRule: ExitRule = { key: "refused", head, text: refused };
  return outcome === "refunding" ? [refusedRule] : [{ key: "issued", head: "If the visa is issued", text: issued }, refusedRule];
}

function openExits(v: Vault, university: string, usdc: (micro: number) => string) {
  // The program releases only once total >= target, and then pays the whole balance.
  const issued =
    v.total_contributed >= v.target
      ? `${usdc(v.total_contributed)} USDC goes to ${university} in one payment.`
      : `Once ${usdc(v.target)} USDC is raised, the whole deposit goes to ${university} in one payment.`;
  const n = v.contributors.length;
  const refused =
    n === 0
      ? "Anyone who pays in gets back their own amount."
      : `${n === 1 ? "The funder below gets" : `Each of the ${n} funders below gets`} back their own amount, ${usdc(v.total_contributed)} USDC in all.`;
  return { issued, refused };
}
