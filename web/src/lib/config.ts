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

/** Registry addresses with display names from the env overrides; unnamed ones show the address. */
export function nameParties(addresses: string[], overrides: Party[]): Party[] {
  return addresses.map((address) => ({ name: overrides.find((o) => o.address === address)?.name ?? address, address }));
}

// Display-name overrides ("Name=address"), and the pickers' fallback when GET /registry fails.
export const UNIVERSITIES = parseParties(process.env.NEXT_PUBLIC_UNIVERSITIES);
export const ATTESTORS = parseParties(process.env.NEXT_PUBLIC_ATTESTORS);

export const explorerTx = (sig: string) => `https://explorer.solana.com/tx/${encodeURIComponent(sig)}?cluster=devnet`;
export const explorerAddress = (a: string) => `https://explorer.solana.com/address/${encodeURIComponent(a)}?cluster=devnet`;
