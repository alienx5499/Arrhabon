import type { Metadata } from "next";
import { EB_Garamond, Source_Serif_4 } from "next/font/google";
import "./globals.css";
import { WalletProvider } from "@/components/wallet";
import { LoginBar } from "@/components/LoginBar";
import { Footer } from "@/components/Footer";

// Bound to the "Newsprint Display" / "Newsprint Reading" token stacks in globals.css.
const display = Source_Serif_4({ subsets: ["latin"], axes: ["opsz"], variable: "--font-newsprint-display" });
const reading = EB_Garamond({ subsets: ["latin"], style: ["normal", "italic"], variable: "--font-newsprint-reading" });

export const metadata: Metadata = {
  title: "Arrhabon",
  description: "Tuition deposits held by a Solana program: released on visa issue, refunded on denial or expiry.",
  icons: { icon: "/favicon.png" },
};

export default function RootLayout({ children }: LayoutProps<"/">) {
  return (
    <html lang="en" className={`${display.variable} ${reading.variable} h-full antialiased`}>
      <body className="flex min-h-full flex-col">
        <WalletProvider>
          <LoginBar />
          <main className="mx-auto w-full max-w-[1240px] flex-1 px-4 py-6 sm:px-6">{children}</main>
          <Footer />
        </WalletProvider>
      </body>
    </html>
  );
}
