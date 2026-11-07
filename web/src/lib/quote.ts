// Aurora price-check inputs: the API's chain ids with readable names, slippage typed as a percent.
import type { SwapStatus } from "./api";

export const CHAINS = [
  { id: "base", name: "Base" },
  { id: "arb", name: "Arbitrum" },
  { id: "eth", name: "Ethereum" },
];

/** "1" or "0.5" percent -> basis points; null outside 0-100 or with more than two decimals. */
export function percentToBps(text: string): number | null {
  const m = /^(\d{1,3})(?:\.(\d{1,2}))?$/.exec(text.trim());
  if (!m) return null;
  const bps = Number(m[1]) * 100 + Number((m[2] ?? "").padEnd(2, "0"));
  return bps <= 10_000 ? bps : null;
}

/** Whether the swap has reached a terminal state and polling should stop. */
export function isFinalStatus(status: SwapStatus | string): boolean {
  return status === "success" || status === "refunded" || status === "failed";
}

/** Plain sentence explaining the current swap status. */
export function swapStatusMessage(status: SwapStatus | string): string {
  switch (status) {
    case "pending":
      return "Waiting for deposit and swap to complete.";
    case "success":
      return "Swap complete. USDC delivered to your Solana address on mainnet.";
    case "refunded":
      return "Swap refunded. Funds returned to your origin address.";
    case "failed":
      return "Swap failed.";
    default:
      return "Waiting for deposit and swap to complete.";
  }
}
