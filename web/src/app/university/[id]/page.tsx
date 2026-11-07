import type { Metadata } from "next";
import { displayName } from "@/lib/config";
import { UniversityView } from "./UniversityView";

type Props = { params: Promise<{ id: string }> };

export async function generateMetadata({ params }: Props): Promise<Metadata> {
  const name = displayName((await params).id);
  return {
    title: `Deposits for ${name} | Arrhabon`,
    description: `Tuition deposits named for ${name}, held by a Solana program: paid to the university when the visa is issued, otherwise refunded to every funder.`,
  };
}

export default async function UniversityPage({ params }: Props) {
  return <UniversityView id={(await params).id} />;
}
