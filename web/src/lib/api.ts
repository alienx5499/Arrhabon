// Typed client for the Arrhabon API; amounts are USDC micro-units, times unix seconds.
import { API_URL } from "./config";

export type VaultStatus = "open" | "released" | "refunding" | "closed";
export type Vault = {
  vault: string;
  student: string;
  university: string;
  attestor: string;
  status: VaultStatus;
  target: number;
  total_contributed: number;
  deadline: number;
  seconds_left: number;
  contributors: { funder: string; amount: number; refunded: boolean }[];
};
export type FunderView = {
  funder: string;
  contributions: {
    vault: string;
    vault_status: VaultStatus | null;
    amount: number;
    refund: "not_due" | "pending" | "refunded" | "unavailable";
  }[];
};
export type UniversityProof = {
  vault: string;
  student: string;
  status: VaultStatus;
  funds_held: number;
  target: number;
  deadline: number;
  attestor: string;
  release_condition: string;
};
export type Health = { status: string; cluster: string; chain: string; crank: unknown; demo_attestor: boolean };
/** Active registry entries; the program stores addresses only. */
export type Registry = { universities: { address: string }[]; attestors: { address: string }[] };
export type UnsignedTx = { unsigned_tx: string; last_valid_block_height: number };
export type Quote = {
  dry: boolean;
  deposit_address: string;
  /** Guaranteed floor: at least this USDC arrives, or the swap refunds. */
  min_usdc_out: number;
  fee: number;
  expires_at: number | string;
};

/** DEMO faucet grant: devnet SOL in lamports and test USDC in micro units (string, may exceed 2^53). */
export type FaucetGrant = { demo: true; notice: string; signature: string; lamports: number; usdc_micro: string };

export class ApiError extends Error {
  constructor(message: string, readonly code: string, readonly status: number) {
    super(message);
  }
}

async function call<T>(path: string, body?: unknown): Promise<T> {
  let res: Response;
  try {
    res = await fetch(`${API_URL}${path}`, {
      method: body === undefined ? "GET" : "POST",
      headers: body === undefined ? undefined : { "Content-Type": "application/json" },
      body: body === undefined ? undefined : JSON.stringify(body),
      cache: "no-store",
    });
  } catch {
    throw new ApiError(`API unreachable at ${API_URL}`, "network", 0);
  }
  const json = await res.json().catch(() => null);
  if (!res.ok) {
    throw new ApiError(json?.error ?? `HTTP ${res.status}`, json?.code ?? "http", res.status);
  }
  return json as T;
}

const seg = encodeURIComponent;

export const api = {
  health: () => call<Health>("/health"),
  registry: () => call<Registry>("/registry"),
  vault: (id: string) => call<Vault>(`/vaults/${seg(id)}`),
  studentVaults: (student: string) => call<Vault[]>(`/students/${seg(student)}/vaults`),
  funder: (funder: string) => call<FunderView>(`/funders/${seg(funder)}`),
  universityProof: (u: string) => call<UniversityProof[]>(`/universities/${seg(u)}/proof`),
  createVault: (b: { student: string; university: string; attestor: string; target: number; deadline: number; nonce: number }) =>
    call<UnsignedTx & { vault: string }>("/vaults", b),
  contribute: (vault: string, b: { funder: string; amount: number }) =>
    call<UnsignedTx>(`/vaults/${seg(vault)}/contributions`, b),
  submit: (tx: string) => call<{ signature: string }>("/tx/submit", { tx }),
  faucet: (wallet: string) => call<FaucetGrant>("/demo/faucet", { wallet }),
  demoAttest: (vault: string, outcome: "issued" | "denied") =>
    call<{ demo: true; notice: string; signature: string }>("/demo/attest", { vault, outcome }),
  quote: (b: {
    vault: string;
    origin_chain: string;
    origin_asset: string;
    origin_amount: string;
    funder: string;
    refund_to: string;
    slippage_bps: number;
  }) => call<Quote>("/funding/quote", b),
};

export const b64ToBytes = (b64: string) => Uint8Array.from(atob(b64), (c) => c.charCodeAt(0));
export const bytesToB64 = (bytes: Uint8Array) => btoa(Array.from(bytes, (b) => String.fromCharCode(b)).join(""));
