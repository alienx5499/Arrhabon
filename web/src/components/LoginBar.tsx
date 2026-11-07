import Link from "next/link";
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
          <span className="net-tag" title="Runs on Solana devnet with test money (tUSD)">Devnet</span>
        </Link>
        <nav aria-label="Main" className="site-links">
          <Link className="inline-block py-1" href="/#how-it-works">How it works</Link>
          <Link className="inline-block py-1" href="/#where-it-can-go">Where it can go</Link>
          <Link className="inline-block py-1" href="/#questions">Questions</Link>
        </nav>
        <div id={ACCOUNT_SLOT_ID} className="account" />
        <HeaderAction />
      </div>
    </header>
  );
}
