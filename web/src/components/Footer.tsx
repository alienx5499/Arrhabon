import Image from "next/image";
import Link from "next/link";
import { PROGRAM_ID, explorerAddress } from "@/lib/config";

/** Site directory: the same pages and facts the app already exposes. */
export function Footer() {
  return (
    <footer className="mx-auto w-full max-w-[1240px] px-4 pb-8 sm:px-6">
      <div className="rule-section directory pt-4 text-sm">
        <div className="flex items-start gap-3">
          <Image src="/token.png" alt="" width={40} height={40} className="h-10 w-10" />
          <div>
            <p className="section-head mb-1">Arrhabon</p>
            <p className="muted">Runs on Solana devnet with test USDC.</p>
          </div>
        </div>
        <nav aria-label="Directory">
          <h2 className="label mb-2">Pages</h2>
          <ul className="flex flex-col gap-1">
            <li><Link className="link" href="/">Front page</Link></li>
            <li><Link className="link" href="/student">Student: create a vault</Link></li>
          </ul>
        </nav>
        <div>
          <h2 className="label mb-2">Funders and universities</h2>
          <p className="muted">Funders open the vault link the student shares. Universities open /university/&lt;their address&gt;.</p>
        </div>
        <div>
          <h2 className="label mb-2">Program</h2>
          <p>
            <a className="link font-mono" href={explorerAddress(PROGRAM_ID)} target="_blank" rel="noreferrer">
              {PROGRAM_ID.slice(0, 4)}…{PROGRAM_ID.slice(-4)} (devnet explorer)
            </a>
          </p>
          <p className="muted mt-1">The visa attestor is the one trusted party.</p>
        </div>
      </div>
    </footer>
  );
}
