// USDC amounts arrive from the API as micro-units (6 decimals); string/bigint math only, no floats.
const DECIMALS = 6;
const SCALE = 10n ** BigInt(DECIMALS);

/** Formats micro-units as "1,234.5 USDC"-style text, keeping at least two decimals. */
export function formatUsdc(micro: number | bigint | string): string {
  const v = BigInt(micro);
  const neg = v < 0n;
  const abs = neg ? -v : v;
  const whole = (abs / SCALE).toLocaleString("en-US");
  const frac = (abs % SCALE).toString().padStart(DECIMALS, "0").replace(/0{1,4}$/, "");
  return `${neg ? "-" : ""}${whole}.${frac.padEnd(2, "0")}`;
}

/** Parses "12.5" into micro-units; null when the text is not a non-negative amount with at most 6 decimals. */
export function parseUsdc(text: string): bigint | null {
  const m = /^(\d+)(?:\.(\d{0,6}))?$/.exec(text.trim().replace(/,/g, ""));
  if (!m) return null;
  return BigInt(m[1]) * SCALE + BigInt((m[2] ?? "").padEnd(DECIMALS, "0"));
}
