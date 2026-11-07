import type { Metadata } from "next";
import type { ReactNode } from "react";
import { Analytics } from "@vercel/analytics/next";
import { Epilogue, IBM_Plex_Mono } from "next/font/google";
import "./globals.css";
import { LoginBar } from "@/components/LoginBar";
import { Footer } from "@/components/Footer";

// Bound to the --font-display / --font-text / --font-mono token stacks in globals.css; Epilogue is variable, 400 to 900.
const epilogue = Epilogue({ subsets: ["latin"], variable: "--font-epilogue" });
const mono = IBM_Plex_Mono({ subsets: ["latin"], weight: "400", variable: "--font-plex-mono" });

const description =
  "A tuition deposit that refunds itself. Family pays into one vault: the university is paid if the visa is issued, and everyone gets their money back if it is refused or the deadline passes.";

// Absolute base for share-image URLs: the set site URL, else the Vercel host, else local dev.
const vercelHost = process.env.VERCEL_PROJECT_PRODUCTION_URL || process.env.VERCEL_URL;
const siteUrl = process.env.NEXT_PUBLIC_SITE_URL?.trim() || (vercelHost ? `https://${vercelHost}` : "http://localhost:3000");

// Icons and share images come from app/icon.svg, app/apple-icon.png and app/opengraph-image.*.
export const metadata: Metadata = {
  metadataBase: new URL(siteUrl),
  title: { default: "Arrhabon: a tuition deposit that refunds itself", template: "%s | Arrhabon" },
  description,
  applicationName: "Arrhabon",
  openGraph: { type: "website", siteName: "Arrhabon", title: "Arrhabon", description },
  twitter: { card: "summary_large_image", title: "Arrhabon", description },
};

export default function RootLayout({ children }: { children: ReactNode }) {
  return (
    <html lang="en" className={`${epilogue.variable} ${mono.variable} h-full antialiased`}>
      <body className="flex min-h-full flex-col">
        <a href="#main" className="skip-link">Skip to content</a>
        <LoginBar />
        {/* Wallet routes (student, vault) mount WalletProvider in their own layout so this page stays free of Privy. */}
        <main id="main" tabIndex={-1} className="wrap flex-1 py-8 outline-none">{children}</main>
        <Footer />
        <Analytics />
      </body>
    </html>
  );
}
