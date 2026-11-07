import Link from "next/link";
import { PRIVY_APP_ID } from "@/lib/config";
import { HeaderAction } from "./HeaderAction";
import { Mark } from "./Mark";

export const ACCOUNT_SLOT_ID = "account-slot";

/** Site header: wordmark, section links, account slot (filled on wallet routes), the one main action. */
export function LoginBar() {
  return (
    <header>
      <div className="wrap site-nav">
        <Link href="/" className="brand">
          <Mark />
          Arrhabon
        </Link>
        <nav aria-label="Main" className="site-links">
          <Link className="inline-block py-1" href="/#how-it-works">How it works</Link>
          <Link className="inline-block py-1" href="/#why-trust-it">Why trust it</Link>
          <Link className="inline-block py-1" href="/#see-it-work">See a live vault</Link>
        </nav>
        <div id={ACCOUNT_SLOT_ID} className="account" />
        <HeaderAction />
      </div>
      <div className="wrap">
        <p className="notice">
          Demo on Solana devnet with test money (tUSD).
          {!PRIVY_APP_ID && process.env.NODE_ENV === "development" &&
            " Wallet login is off: set NEXT_PUBLIC_PRIVY_APP_ID (see .env.example) to create vaults and contribute."}
        </p>
      </div>
    </header>
  );
}
