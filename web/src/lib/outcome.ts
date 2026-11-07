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
    body: "Funders can add money until the deadline. If the visa is issued, the money goes to the university. If the visa is denied or the deadline passes first, every funder gets their money back automatically.",
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
