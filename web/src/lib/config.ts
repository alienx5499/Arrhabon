// Public runtime config; every value comes from NEXT_PUBLIC_* env vars inlined at build time.
export type Party = { name: string; address: string };

export const PRIVY_APP_ID = process.env.NEXT_PUBLIC_PRIVY_APP_ID?.trim() ?? "";
export const API_URL = (process.env.NEXT_PUBLIC_API_URL?.trim() || "http://localhost:8080").replace(/\/$/, "");
export const SOLANA_RPC_URL = process.env.NEXT_PUBLIC_SOLANA_RPC_URL?.trim() || "https://api.devnet.solana.com";
export const PROGRAM_ID = process.env.NEXT_PUBLIC_PROGRAM_ID?.trim() || "6p3YF6s88hUk6t2mtNQ792gDBkMUxQxYRTCYafqGj8JE";

/** Parses "Name=address,Name2=address2"; entries without "=" use the address as the name. */
export function parseParties(raw: string | undefined): Party[] {
  return (raw ?? "")
    .split(",")
    .map((e) => e.trim())
    .filter(Boolean)
    .map((e) => {
      const i = e.lastIndexOf("=");
      return i < 0 ? { name: e, address: e } : { name: e.slice(0, i).trim(), address: e.slice(i + 1).trim() };
    });
}

/** Parses a JSON object of address -> display name; anything else (unset, invalid, non-string names) yields {}. */
export function parseNameMap(raw: string | undefined): Record<string, string> {
  try {
    const m: unknown = JSON.parse(raw ?? "");
    if (!m || typeof m !== "object" || Array.isArray(m)) return {};
    return Object.fromEntries(Object.entries(m).filter((e): e is [string, string] => typeof e[1] === "string"));
  } catch {
    return {};
  }
}

export const shortAddress = (a: string) => (a.length > 12 ? `${a.slice(0, 4)}…${a.slice(-4)}` : a);

// Display-name overrides ("Name=address"), and the pickers' fallback when GET /registry fails.
export const UNIVERSITIES = parseParties(process.env.NEXT_PUBLIC_UNIVERSITIES);
export const ATTESTORS = parseParties(process.env.NEXT_PUBLIC_ATTESTORS);
// The registry API stores addresses only, so names come from this JSON map first, then the lists above.
export const REGISTRY_NAMES = parseNameMap(process.env.NEXT_PUBLIC_REGISTRY_NAMES);

/** Display name for a registry address: the JSON map, then the Name=address lists, else the shortened address. */
export function displayName(address: string, names = REGISTRY_NAMES, parties: Party[] = [...UNIVERSITIES, ...ATTESTORS]): string {
  return names[address] ?? parties.find((p) => p.address === address)?.name ?? shortAddress(address);
}

/** Registry addresses with display names for the pickers. */
export function nameParties(addresses: string[], overrides: Party[], names = REGISTRY_NAMES): Party[] {
  return addresses.map((address) => ({ name: displayName(address, names, overrides), address }));
}

export const explorerTx = (sig: string) => `https://explorer.solana.com/tx/${encodeURIComponent(sig)}?cluster=devnet`;
export const explorerAddress = (a: string) => `https://explorer.solana.com/address/${encodeURIComponent(a)}?cluster=devnet`;

export type ShowcaseVault = { label: string; id: string };
const BASE58_ADDRESS = /^[1-9A-HJ-NP-Za-km-z]{32,44}$/;

/** Parses the JSON list of showcase vaults; unset is empty, anything malformed fails the build loudly. */
export function parseShowcase(raw: string | undefined): ShowcaseVault[] {
  if (!raw?.trim()) return [];
  const list: unknown = JSON.parse(raw);
  if (!Array.isArray(list)) throw new Error("NEXT_PUBLIC_SHOWCASE_VAULTS must be a JSON array");
  return list.map((v) => {
    const label = typeof v?.label === "string" ? v.label.trim() : "";
    const id = typeof v?.id === "string" ? v.id.trim() : "";
    if (!label || !BASE58_ADDRESS.test(id)) throw new Error(`Bad showcase vault: ${JSON.stringify(v)}`);
    return { label, id };
  });
}

/** An optional public http(s) link from env; unset is "", any other scheme fails loudly. */
export function httpUrl(raw: string | undefined, name: string): string {
  const url = raw?.trim() ?? "";
  if (url && !/^https?:\/\//i.test(url)) throw new Error(`${name} must start with http:// or https://`);
  return url;
}

export const SHOWCASE_VAULTS = parseShowcase(process.env.NEXT_PUBLIC_SHOWCASE_VAULTS);
export const REPO_URL = httpUrl(process.env.NEXT_PUBLIC_REPO_URL, "NEXT_PUBLIC_REPO_URL");
export const DEMO_VIDEO_URL = httpUrl(process.env.NEXT_PUBLIC_DEMO_VIDEO_URL, "NEXT_PUBLIC_DEMO_VIDEO_URL");
