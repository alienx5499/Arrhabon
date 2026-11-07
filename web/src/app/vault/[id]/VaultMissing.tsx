import Link from "next/link";

/** Calm "no such vault" view, shared by the server 404 and the client fallback. */
export function VaultMissing() {
  return (
    <section className="flex flex-col items-start gap-5 py-8" aria-labelledby="vault-missing">
      <h1 id="vault-missing" className="page-title">This vault doesn&apos;t exist.</h1>
      <p className="lead story">The link may be incomplete or mistyped. Ask the student to send it again.</p>
      <div className="flex flex-wrap gap-3">
        <Link className="btn-primary" href="/">Back to the front page</Link>
        <Link className="btn-quiet" href="/student">Open a vault</Link>
      </div>
    </section>
  );
}
