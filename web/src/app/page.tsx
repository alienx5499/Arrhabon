import type { ReactNode } from "react";
import Link from "next/link";
import { ArtSlot } from "@/components/ArtSlot";
import { DEMO_VIDEO_URL, REPO_URL, SHOWCASE_VAULTS, UNIVERSITIES } from "@/lib/config";
import "./landing.css";

const vaultHref = (id: string) => `/vault/${encodeURIComponent(id)}`;
const university = UNIVERSITIES[0];
const liveVaultHref = SHOWCASE_VAULTS[0] ? vaultHref(SHOWCASE_VAULTS[0].id) : "#see-it-work";

const STEPS = [
  { title: "Open a vault", text: "The student names the university, the amount and a deadline, then shares the vault link." },
  { title: "Family chips in", text: "Parents, relatives and the student pay in from the link. Every payment is kept with the name of who made it." },
  { title: "Visa decision", text: "The registered attestor confirms whether the visa was issued or refused." },
  { title: "Issued: paid to the university", text: "The whole deposit goes to the university in one payment." },
  { title: "Denied or late: everyone refunded", text: "If the visa is refused, or nobody answers by the deadline, each funder gets back exactly what they paid." },
];

const FAQ: { q: string; a: ReactNode }[] = [
  {
    q: "Who confirms the visa?",
    a: "A registered attestor: a party on the program's list whose signed answer is the only one the vault accepts. This demo uses a demo signer, and every page where it appears says so.",
  },
  {
    q: "Can anyone take the money?",
    a: "No. The program that holds the deposit has two ways to pay out: the whole amount to the named university after an issued visa, or each payment back to the person who made it. Nobody can send it anywhere else.",
  },
  {
    q: "What if nobody decides?",
    a: "When the deadline passes without an answer, the vault switches to refunds. Anyone can start them, and each funder gets back what they paid.",
  },
  {
    q: "What does it cost?",
    a: "Arrhabon takes no fee. Each payment carries a small network fee, usually well under a cent, and in the demo even that is paid with test money.",
  },
  {
    q: "Is this real money?",
    a: "No. The demo runs on test money on Solana devnet. The dollars are tUSD, a test dollar with no value.",
  },
  {
    q: "Is it audited?",
    a: "No. Arrhabon is a prototype and nobody has audited it. Please do not use it for a real deposit yet.",
  },
  {
    q: "Where is the code?",
    a: REPO_URL ? (
      <>
        Every line is open to read: <a href={REPO_URL}>see the code</a>.
      </>
    ) : (
      "The code is public. The link is in the project submission."
    ),
  },
];

export default function Home() {
  return (
    <div className="lp">
      <section className="lp-hero" aria-labelledby="hero-title">
        <h1 id="hero-title">A tuition deposit that refunds itself.</h1>
        <div className="lp-hero-copy">
          <p className="lp-lead">
            Your family pays into one vault. If the visa is issued, the university gets the deposit. If it is refused,
            everyone who paid gets their money back.
          </p>
          <div className="lp-actions">
            <Link className="lp-btn lp-btn-primary" href="/student">Open a vault</Link>
            <Link className="lp-btn lp-btn-quiet" href={liveVaultHref}>See a live vault</Link>
          </div>
        </div>
        <ArtSlot
          className="lp-hero-art"
          src="/art/hero.webp"
          alt="A gold coin under a glass bell jar before a stone gateway, a student with a suitcase beside it and a dashed path leading home."
          ratio="16 / 9"
          sizes="(max-width: 760px) 100vw, 60vw"
          priority
        />
      </section>

      <section className="lp-field" aria-labelledby="exits-title">
        <h2 id="exits-title">The money has two ways out. Nobody can add a third.</h2>
        <div className="lp-exits">
          <div>
            <h3>Visa issued</h3>
            <p>The attestor confirms. The whole deposit goes to the university in one payment.</p>
          </div>
          <div>
            <h3>Visa refused, or no answer by the deadline</h3>
            <p>Every funder gets back exactly what they paid. Anyone can start the refund.</p>
          </div>
        </div>
      </section>

      <section className="lp-section" id="how-it-works" aria-labelledby="how-title">
        <h2 id="how-title">How it works</h2>
        <ArtSlot
          className="lp-how-art"
          src="/art/how-it-works.webp"
          alt="Five scenes in a row: a student sets down a glass jar, three homes send coins into it, a sealed letter arrives, a coin rolls through the gateway, and coins return to each home."
          ratio="3 / 1"
          sizes="(max-width: 1240px) 100vw, 1200px"
        />
        <ol className="lp-steps">
          {STEPS.map((s) => (
            <li key={s.title}>
              <h3>{s.title}</h3>
              <p>{s.text}</p>
            </li>
          ))}
        </ol>
      </section>

      <section className="lp-section" id="why-trust-it" aria-labelledby="trust-title">
        <h2 id="trust-title">Why trust it</h2>
        <div className="lp-faq">
          {FAQ.map((f) => (
            <details key={f.q}>
              <summary>{f.q}</summary>
              <p>{f.a}</p>
            </details>
          ))}
        </div>
      </section>

      <section className="lp-section" id="see-it-work" aria-labelledby="see-title">
        <h2 id="see-title">See it work</h2>
        <ul className="lp-cards">
          {SHOWCASE_VAULTS.map((v) => (
            <li key={v.id}>
              <Link className="lp-card" href={vaultHref(v.id)}>
                <span className="lp-card-title">{v.label} vault</span>
                <span className="lp-card-note">Every payment, the decision and where the money went.</span>
              </Link>
            </li>
          ))}
          {university && (
            <li>
              <Link className="lp-card" href={`/university/${encodeURIComponent(university.address)}`}>
                <span className="lp-card-title">University proof page</span>
                <span className="lp-card-note">What a university sees: every deposit that names it, and where each one stands.</span>
              </Link>
            </li>
          )}
          {DEMO_VIDEO_URL && (
            <li className="lp-card-video">
              <a className="lp-card" href={DEMO_VIDEO_URL}>
                <ArtSlot src="/art/poster.webp" alt="" ratio="16 / 9" sizes="(max-width: 760px) 100vw, 40vw" />
                <span className="lp-card-title">Watch the demo</span>
                <span className="lp-card-note">A short film of a deposit paid in, decided and settled.</span>
              </a>
            </li>
          )}
        </ul>
        <div className="lp-note">
          <h3>How to get test money</h3>
          <p>
            Reading a vault needs no money. To pay in yourself, log in with your email and the app makes a wallet for
            you. Copy its address, take a little free test SOL for network fees from{" "}
            <a href="https://faucet.solana.com">faucet.solana.com</a> (choose devnet), then ask for tUSD, the
            demo&apos;s test dollar,{" "}
            {REPO_URL ? <>by opening an issue on <a href={REPO_URL}>the code repository</a></> : "through the project submission"}.
          </p>
        </div>
      </section>
    </div>
  );
}
