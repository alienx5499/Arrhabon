"use client";
// Wallet seam: Privy when an app id is configured, read-only otherwise, so pages never call Privy hooks directly.
// Privy and Solana Kit live in wallet-privy.tsx and load only when an app id is set, and only on routes whose layout mounts WalletProvider.
import { createContext, lazy, Suspense, useContext, type ReactNode } from "react";
import { PRIVY_APP_ID } from "@/lib/config";

export type Wallet = {
  /** False in read-only mode (no Privy app id). */
  enabled: boolean;
  ready: boolean;
  address: string | null;
  login: () => void;
  logout: () => void;
  /** Signs wire-format transaction bytes with the user's Solana wallet on devnet. */
  sign: (tx: Uint8Array) => Promise<Uint8Array>;
};

const readOnly: Wallet = {
  enabled: false,
  ready: true,
  address: null,
  login: () => {},
  logout: () => {},
  sign: () => Promise.reject(new Error("Wallet login is off in this demo, so transactions cannot be signed")),
};
/** Shown while the Privy chunk loads: login exists but is not ready yet. */
const loading: Wallet = { ...readOnly, enabled: true, ready: false };

export const WalletContext = createContext<Wallet>(readOnly);
export const useWallet = () => useContext(WalletContext);

const PrivyWallet = PRIVY_APP_ID ? lazy(() => import("./wallet-privy")) : null;

export function WalletProvider({ children }: { children: ReactNode }) {
  if (!PrivyWallet) return <WalletContext.Provider value={readOnly}>{children}</WalletContext.Provider>;
  return (
    <Suspense fallback={<WalletContext.Provider value={loading}>{children}</WalletContext.Provider>}>
      <PrivyWallet appId={PRIVY_APP_ID}>{children}</PrivyWallet>
    </Suspense>
  );
}
