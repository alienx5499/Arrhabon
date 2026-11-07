import type { Metadata, ResolvingMetadata } from "next";
import { notFound } from "next/navigation";
import { cache } from "react";
import { ApiError, api } from "@/lib/api";
import { displayName } from "@/lib/config";
import { withinMs } from "@/lib/withinMs";
import { formatUsdc } from "@/lib/usdc";
import { VaultView } from "./VaultView";

type Props = { params: Promise<{ id: string }> };

// One read per request, shared by the metadata and the page. A sleeping API must not hold up the page:
// after 3 s (or on any other failure) the client loads the vault itself.
const loadVault = cache((id: string) =>
  withinMs(
    api.vault(id).catch((e) => {
      if (e instanceof ApiError && e.status === 404) return "missing" as const;
      throw e;
    }),
    3000,
  ),
);

export async function generateMetadata({ params }: Props, parent: ResolvingMetadata): Promise<Metadata> {
  const v = await loadVault((await params).id);
  const up = await parent;
  const what = "A tuition deposit held by a Solana program: paid to the university if the visa is issued, refunded to every funder if it is denied or the deadline passes.";
  if (!v || v === "missing") return { title: "Deposit vault", description: what };
  const title = `Deposit for ${displayName(v.university)}`;
  const share = `${formatUsdc(v.total_contributed)} of ${formatUsdc(v.target)} USDC raised. Refunds itself if the visa is refused.`;
  return {
    title,
    description: `${what} Raised ${formatUsdc(v.total_contributed)} of ${formatUsdc(v.target)} USDC.`,
    // A segment's openGraph replaces the root one whole, so the site name, type and share image are restated.
    openGraph: { type: "website", siteName: "Arrhabon", title, description: share, images: up.openGraph?.images },
    twitter: { card: "summary_large_image", title, description: share, images: up.twitter?.images },
  };
}

export default async function VaultPage({ params }: Props) {
  const { id } = await params;
  const v = await loadVault(id);
  if (v === "missing") notFound();
  return <VaultView id={id} initial={v} />;
}
