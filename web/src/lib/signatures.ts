// Recent transactions touching an address, read straight from the Solana RPC (newest first).
import { SOLANA_RPC_URL } from "./config";

export type TxRow = { signature: string; blockTime: number | null; failed: boolean };
type RpcRow = { signature: string; blockTime?: number | null; err: unknown };

export async function recentSignatures(address: string, rpcUrl = SOLANA_RPC_URL, fetchFn: typeof fetch = fetch): Promise<TxRow[]> {
  const res = await fetchFn(rpcUrl, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ jsonrpc: "2.0", id: 1, method: "getSignaturesForAddress", params: [address, { limit: 10, commitment: "confirmed" }] }),
    cache: "no-store",
  });
  if (!res.ok) throw new Error(`RPC HTTP ${res.status}`);
  const json = (await res.json()) as { result?: RpcRow[]; error?: { message?: string } };
  if (json.error || !json.result) throw new Error(`RPC error: ${json.error?.message ?? "no result"}`);
  return json.result.map((r) => ({ signature: r.signature, blockTime: r.blockTime ?? null, failed: r.err != null }));
}
