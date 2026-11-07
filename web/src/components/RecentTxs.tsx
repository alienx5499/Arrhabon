// Pure display of a vault's recent transactions (newest first) with devnet explorer links.
import { formatDeadline } from "@/lib/countdown";
import type { TxRow } from "@/lib/signatures";
import { Card } from "./parts";
import { TxLink } from "./ui";

export function RecentTxs({ rows, error, onRetry }: { rows: TxRow[] | null; error: unknown; onRetry?: () => void }) {
  return (
    <Card title="Recent transactions">
      {/* The list is extra: a failed RPC read gets a quiet note, and rows from an earlier read stay. */}
      {error && !rows ? (
        <p className="muted text-sm">
          Couldn&apos;t load recent transactions right now.
          {onRetry && (
            <>
              {" "}
              <button type="button" className="link" onClick={onRetry}>Try again</button>
            </>
          )}
        </p>
      ) : (
        rows?.length === 0 && <p className="text-sm">No transactions yet.</p>
      )}
      <ul className="listing text-sm">
        {rows?.map((r) => (
          <li key={r.signature} className="flex flex-wrap gap-3">
            <TxLink sig={r.signature} label={r.failed ? "Failed" : "Transaction"} />
            {r.blockTime && <span className="muted">{formatDeadline(r.blockTime)}</span>}
          </li>
        ))}
      </ul>
    </Card>
  );
}
