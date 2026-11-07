// Turns a thrown error into a plain message; raw API/RPC text goes to `detail` for a "Details" toggle.
import { ApiError } from "./api";

export type Problem = { message: string; detail: string | null; retry: boolean };

export function classifyProblem(e: unknown): Problem {
  const raw = e instanceof Error ? e.message : String(e);
  if (e instanceof ApiError) {
    const detail = `${e.status || "no response"} ${e.code}: ${raw}`;
    // 502-504: the free host's proxy answers while the service starts.
    if (e.code === "network" || [502, 503, 504].includes(e.status)) {
      return { message: "The vault service is starting up. This can take up to a minute.", detail, retry: true };
    }
    if (e.status === 404) return { message: "Not found. Check that the link is complete.", detail, retry: false };
    return { message: "The vault service could not do that. Open Details for the reason.", detail, retry: false };
  }
  // fetch() rejects with TypeError when the network is down; signatures.ts prefixes RPC failures.
  if (e instanceof TypeError || raw.startsWith("RPC")) {
    return { message: "The Solana network did not answer.", detail: raw, retry: true };
  }
  // Errors raised by this app are already written for people.
  return { message: raw, detail: null, retry: false };
}
