import type { ReactNode } from "react";
import { WalletProvider } from "@/components/wallet";
import { Account } from "@/components/Account";

export default function VaultLayout({ children }: { children: ReactNode }) {
  return (
    <WalletProvider>
      <Account />
      {children}
    </WalletProvider>
  );
}
