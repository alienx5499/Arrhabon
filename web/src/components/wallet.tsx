"use client";
// Wallet seam: Privy when an app id is configured, read-only otherwise, so pages never call Privy hooks directly.
import { createContext, useContext, useMemo, type ReactNode } from "react";
import { PrivyProvider, usePrivy } from "@privy-io/react-auth";
import { useSignTransaction, useWallets } from "@privy-io/react-auth/solana";
import { createSolanaRpc, createSolanaRpcSubscriptions } from "@solana/kit";
import { PRIVY_APP_ID, SOLANA_RPC_URL } from "@/lib/config";

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
  sign: () => Promise.reject(new Error("Read-only mode: set NEXT_PUBLIC_PRIVY_APP_ID to sign transactions")),
};

const WalletContext = createContext<Wallet>(readOnly);
export const useWallet = () => useContext(WalletContext);

function PrivyWallet({ children }: { children: ReactNode }) {
  const { ready, authenticated, login, logout } = usePrivy();
  const { wallets } = useWallets();
  const { signTransaction } = useSignTransaction();
  const wallet = authenticated ? wallets[0] : undefined;
  const value = useMemo<Wallet>(
    () => ({
      enabled: true,
      ready,
      address: wallet?.address ?? null,
      login,
      logout: () => void logout(),
      sign: async (transaction) => {
        if (!wallet) throw new Error("Log in to sign");
        const { signedTransaction } = await signTransaction({ transaction, wallet, chain: "solana:devnet" });
        return signedTransaction;
      },
    }),
    [ready, wallet, login, logout, signTransaction],
  );
  return <WalletContext.Provider value={value}>{children}</WalletContext.Provider>;
}

export function WalletProvider({ children }: { children: ReactNode }) {
  const rpcs = useMemo(
    () => ({
      "solana:devnet": {
        rpc: createSolanaRpc(SOLANA_RPC_URL),
        rpcSubscriptions: createSolanaRpcSubscriptions(SOLANA_RPC_URL.replace(/^http/, "ws")),
        blockExplorerUrl: "https://explorer.solana.com/?cluster=devnet",
      },
    }),
    [],
  );
  if (!PRIVY_APP_ID) return <WalletContext.Provider value={readOnly}>{children}</WalletContext.Provider>;
  return (
    <PrivyProvider
      appId={PRIVY_APP_ID}
      config={{
        loginMethods: ["email"],
        appearance: { walletChainType: "solana-only" },
        embeddedWallets: { solana: { createOnLogin: "users-without-wallets" } },
        solana: { rpcs },
      }}
    >
      <PrivyWallet>{children}</PrivyWallet>
    </PrivyProvider>
  );
}
