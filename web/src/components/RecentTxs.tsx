// Pure display of a vault's recent transactions (newest first) with devnet explorer links.
import type { TxRow } from "@/lib/signatures";
import { Card, ErrorText, TxLink } from "./ui";

export function RecentTxs({ rows, error }: { rows: TxRow[] | null; error: unknown }) {
  return (
    <Card title="Recent transactions">
      <ErrorText error={error} />
      {rows?.length === 0 && <p className="text-sm">No transactions yet.</p>}
      <ul className="listing text-sm">
        {rows?.map((r) => (
          <li key={r.signature} className="flex flex-wrap gap-3">
            <TxLink sig={r.signature} label={r.failed ? "Failed" : "Transaction"} />
            {r.blockTime && <span className="muted">{new Date(r.blockTime * 1000).toLocaleString()}</span>}
          </li>
        ))}
      </ul>
    </Card>
  );
}
