"use client";
import Image from "next/image";
import Link from "next/link";
import { useWallet } from "./wallet";
import { AddressLink } from "./ui";

export function LoginBar() {
  const w = useWallet();
  return (
    <header className="mx-auto w-full max-w-[1240px] px-4 sm:px-6">
      <nav className="flex flex-wrap items-center gap-x-5 gap-y-2 py-3 text-sm" aria-label="Main">
        <Link href="/student" className="link field-label">Student</Link>
        <span className="label muted">devnet</span>
        <div className="ml-auto flex items-center gap-3">
          {!w.enabled ? (
            <span className="label">Read-only</span>
          ) : w.address ? (
            <>
              <AddressLink address={w.address} />
              <button className="btn-secondary" onClick={w.logout}>Log out</button>
            </>
          ) : (
            <button className="btn" onClick={w.login} disabled={!w.ready}>Log in</button>
          )}
        </div>
      </nav>
      <div className="rule-hair flex items-center justify-center gap-3 py-4 sm:gap-5">
        <Image src="/token.png" alt="" width={64} height={64} className="h-10 w-10 sm:h-16 sm:w-16" priority />
        <Link href="/" className="masthead" aria-label="Arrhabon, front page">Arrhabon</Link>
      </div>
      <p className="rule-hair border-b border-b-rule py-1.5 text-center compact italic muted">
        Tuition deposits held by a Solana program: released on visa issue, refunded on denial or expiry.
      </p>
      {!w.enabled && (
        <p role="status" className="border-b border-b-rule py-2 text-center text-sm">
          <span className="label label-accent mr-2">Read-only mode:</span>
          wallet login is off. Set NEXT_PUBLIC_PRIVY_APP_ID (see .env.example) to create vaults and contribute.
        </p>
      )}
      <div className="rule-section" />
    </header>
  );
}
