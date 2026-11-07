"use client";
// Privy-backed wallet; loaded lazily by WalletProvider in wallet.tsx so pages without a wallet never ship Privy or Solana Kit.
import { useMemo, type ReactNode } from "react";
import { PrivyProvider, usePrivy } from "@privy-io/react-auth";
import { useSignTransaction, useWallets } from "@privy-io/react-auth/solana";
import { createSolanaRpc, createSolanaRpcSubscriptions } from "@solana/kit";
import { SOLANA_RPC_URL } from "@/lib/config";
import { WalletContext, type Wallet } from "./wallet";

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

export default function PrivyWalletProvider({ appId, children }: { appId: string; children: ReactNode }) {
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
  return (
    <PrivyProvider
      appId={appId}
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
