import type { ReactNode } from "react";
import { WalletProvider } from "@/components/wallet";
import { Account } from "@/components/Account";

export default function StudentLayout({ children }: { children: ReactNode }) {
  return (
    <WalletProvider>
      <Account />
      {children}
    </WalletProvider>
  );
}
