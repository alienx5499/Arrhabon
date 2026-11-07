import type { Metadata } from "next";
import type { ReactNode } from "react";
import { WalletProvider } from "@/components/wallet";
import { Account } from "@/components/Account";

export const metadata: Metadata = {
  title: "Open a vault",
  description:
    "Open a tuition deposit vault: name the university, the amount and a deadline, then share the link with your family. Paid to the university if the visa is issued, refunded to every funder if not.",
};

export default function StudentLayout({ children }: { children: ReactNode }) {
  return (
    <WalletProvider>
      <Account />
      {children}
    </WalletProvider>
  );
}
