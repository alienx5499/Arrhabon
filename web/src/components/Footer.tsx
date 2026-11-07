import Link from "next/link";
import { PROGRAM_ID, UNIVERSITIES, explorerAddress } from "@/lib/config";
import { Mark } from "./Mark";

const university = UNIVERSITIES[0];

/** Site directory: the same pages and facts the app already exposes. */
export function Footer() {
  return (
    <footer className="footer">
      <div className="wrap directory">
        <div>
          <p className="footer-brand">
            <Mark size={32} />
            Arrhabon
          </p>
          <p className="muted mt-3">Runs on Solana devnet with test money (tUSD).</p>
        </div>
        <nav aria-label="Directory">
          <p className="label label-accent mb-2">Pages</p>
          <ul className="flex flex-col">
            <li><Link className="link inline-block py-1" href="/">Home</Link></li>
            <li><Link className="link inline-block py-1" href="/student">Student: open a vault</Link></li>
          </ul>
        </nav>
        <div>
          <p className="label label-accent mb-2">Funders and universities</p>
          <p className="muted">Funders open the vault link the student shares.</p>
          {university ? (
            <p className="mt-1">
              <Link className="link inline-block py-1" href={`/university/${encodeURIComponent(university.address)}`}>
                See a university&apos;s page
              </Link>
            </p>
          ) : (
            <p className="muted">Universities: open your page from the link we send you.</p>
          )}
        </div>
        <div>
          <p className="label label-accent mb-2">Program</p>
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
