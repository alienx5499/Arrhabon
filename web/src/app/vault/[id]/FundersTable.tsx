"use client";
// "Who has paid": every funder from the vault account, their share, where their money is now, and the total.
import type { Vault } from "@/lib/api";
import { displayName, shortAddress } from "@/lib/config";
import { funderTag, shareOf, type Outcome, type Tag } from "@/lib/outcome";
import { formatUsdc } from "@/lib/usdc";
import { AddressLink } from "@/components/ui";

const ROW_TAG: Record<ReturnType<typeof funderTag>, Tag> = {
  held: "open",
  "refund pending": "refunding",
  refunded: "refunded",
  "paid to the university": "released",
};

export function FundersTable({ vault: v, outcome, me }: { vault: Vault; outcome: Outcome; me: string | null }) {
  return (
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
              const you = c.funder === me ? ", you" : "";
              const tag = <span className={`tag tag-${ROW_TAG[where]}`}>{where[0].toUpperCase() + where.slice(1)}</span>;
              return (
                <tr key={c.funder}>
                  {/* An unnamed funder is shown once, as the address link; on phones the status sits under it. */}
                  <td className={named ? "break-words" : "whitespace-nowrap"}>
                    {named ? name : <AddressLink address={c.funder} />}
                    {you}
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
          <tfoot>
            <tr>
              <td>Total</td>
              <td className="hide-sm" />
              <td className="r">{formatUsdc(v.total_contributed)}</td>
              <td className="r">100%</td>
              <td className="hide-sm" />
            </tr>
          </tfoot>
        </table>
      )}
    </section>
  );
}
