import type { Metadata } from "next";
import { api } from "@/lib/api";
import { displayName } from "@/lib/config";
import { withinMs } from "@/lib/withinMs";
import { formatUsdc } from "@/lib/usdc";
import { VaultView } from "./VaultView";

type Props = { params: Promise<{ id: string }> };

export async function generateMetadata({ params }: Props): Promise<Metadata> {
  const { id } = await params;
  // A sleeping API must not hold up the page; fall back to generic text.
  const v = await withinMs(api.vault(id), 3000);
  const what = "A tuition deposit held by a Solana program: paid to the university if the visa is issued, refunded to every funder if it is denied or the deadline passes.";
  if (!v) return { title: "Deposit vault | Arrhabon", description: what };
  return {
    title: `Deposit for ${displayName(v.university)} | Arrhabon`,
    description: `${what} Raised ${formatUsdc(v.total_contributed)} of ${formatUsdc(v.target)} USDC.`,
  };
}

export default async function VaultPage({ params }: Props) {
  return <VaultView id={(await params).id} />;
}
