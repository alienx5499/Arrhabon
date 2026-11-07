// Aurora price-check inputs: the API's chain ids with readable names, slippage typed as a percent.
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
