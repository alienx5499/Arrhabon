import Link from "next/link";
import { DEMO_VIDEO_URL, PROGRAM_ID, REPO_URL, SHOWCASE_VAULTS, UNIVERSITIES, explorerAddress, shortAddress } from "@/lib/config";

const university = UNIVERSITIES[0];

/** Site directory on navy, ending on the name set across the full width. */
export function Footer() {
  return (
    <footer className="foot">
      <div className="wrap">
        <div className="foot-cols">
          <div>
            <span>The name</span>
            <span>Arrhabon is the ancient Greek word for earnest money: a deposit returned or kept by fixed rules.</span>
          </div>
          <nav aria-label="Directory">
            <span>Pages</span>
            <Link href="/#how-it-works">How it works</Link>
            <Link href="/#where-it-can-go">Where it can go</Link>
            <Link href="/student">Open a vault</Link>
            <Link href="/faucet">Test funds</Link>
            {SHOWCASE_VAULTS.map((v) => (
              <Link key={v.id} href={`/vault/${encodeURIComponent(v.id)}`}>{v.label} vault</Link>
            ))}
            {university && <Link href={`/university/${encodeURIComponent(university.address)}`}>A university&apos;s page</Link>}
            {DEMO_VIDEO_URL && <a href={DEMO_VIDEO_URL}>Watch the demo</a>}
          </nav>
          <div>
            <span>Program</span>
            <a className="font-mono" href={explorerAddress(PROGRAM_ID)} target="_blank" rel="noreferrer">
              {shortAddress(PROGRAM_ID)} on devnet
            </a>
            {REPO_URL && <a href={REPO_URL}>The code</a>}
            <span>Not audited. The visa attestor is the one trusted party.</span>
          </div>
        </div>
        <div className="foot-name">
          <svg viewBox="0 0 1000 200" role="img" aria-label="Arrhabon">
            <text x="0" y="168" fontWeight="900" fontSize="232" textLength="1000" lengthAdjust="spacingAndGlyphs" style={{ fontFamily: "var(--font-display)" }}>
              Arrhabon
            </text>
          </svg>
          <p className="foot-legal">Runs on Solana devnet with test money (tUSD). Illustrations are the project&apos;s own.</p>
        </div>
      </div>
    </footer>
  );
}
