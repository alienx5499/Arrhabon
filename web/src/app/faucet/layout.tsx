import type { Metadata } from "next";
import type { ReactNode } from "react";
import { WalletProvider } from "@/components/wallet";
import { Account } from "@/components/Account";

export const metadata: Metadata = {
  title: "Test funds",
  description:
    "Get devnet SOL for fees and test USDC to fund vaults on Solana devnet, worth nothing.",
};

export default function FaucetLayout({ children }: { children: ReactNode }) {
  return (
    <WalletProvider>
      <Account />
      {children}
    </WalletProvider>
  );
}

