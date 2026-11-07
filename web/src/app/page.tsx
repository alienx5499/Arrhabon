import type { ReactNode } from "react";
import Link from "next/link";
import { getImageProps } from "next/image";
import { ArtSlot } from "@/components/ArtSlot";
import { CountUp } from "@/components/CountUp";
import { DeparturesBoard } from "@/components/DeparturesBoard";
import { VaultPlate } from "@/components/VaultPlate";
import { REPO_URL, SHOWCASE_VAULTS } from "@/lib/config";
import "./landing.css";

const liveVault = SHOWCASE_VAULTS[0];

const STAIRS = [
  { head: "First, the student opens a vault.", text: "They name a registered university, a registered visa attestor, the deposit in USDC and a deadline up to a year ahead." },
  { head: "Then the family pays in.", text: "Parents and relatives pay from the shared link, from 1 USDC. Every payment is kept under the name of who made it." },
  { head: "The visa letter arrives.", text: "The attestor confirms on chain whether the visa was issued or refused. Nobody else can." },
  { head: "The money leaves one way.", text: "Through the gate to the university, or back down the road to every funder." },
];

const QUESTIONS: { q: string; a: ReactNode }[] = [
  {
    q: "Who decides that the visa was issued?",
    a: "The attestor named when the vault was opened, from a registry the student cannot add to. In production that is the university's admissions office together with the student, or a licensed agent. A dishonest attestor is the one risk the program cannot remove, and it can never send the money to itself.",
  },
  {
    q: "What if nobody answers?",
    a: "When the deadline passes with no decision, anyone can switch the vault to refunding and trigger each refund. My own crank does it automatically; a funder can do it without me.",
  },
  {
    q: "Is this real money?",
    a: "Not yet. The program runs on Solana devnet with tUSD, a test dollar worth nothing. Nothing here has been audited, and the program can still be upgraded.",
  },
  {
    q: "What does it cost?",
    a: "Arrhabon takes no fee. Each payment carries a small network fee, usually well under a cent, and in the demo even that is paid with test money.",
  },
  {
    q: "Can relatives pay from another chain?",
    a: "A relative holding USDC elsewhere, for example on Base, can get a price quote through Aurora that shows the minimum that would arrive on Solana and the fee.",
  },
  {
    q: "How do I get test money?",
    a: (
      <>
        Reading a vault needs no money. To pay in, log in with your email and the app makes a wallet for you. Take a
        little free test SOL for fees from <a href="https://faucet.solana.com">faucet.solana.com</a> (choose devnet), then
        ask for tUSD{" "}
        {REPO_URL ? <>by opening an issue on <a href={REPO_URL}>the code repository</a></> : "through the project submission"}.
      </>
    ),
  },
];

/** The poster: the wide cover on desktop, the taller hero crop on phones. */
function Poster() {
  const common = { alt: "A student with a suitcase stands on a dusty road. Far ahead, a stone gateway holds a gold coin under a glass bell jar, and a sealed letter is on its way.", sizes: "100vw", quality: 90, priority: true };
  const { props: { srcSet: phone } } = getImageProps({ ...common, src: "/art/hero.webp", width: 2400, height: 1350 });
  const { props: wide } = getImageProps({ ...common, src: "/art/cover.webp", width: 2400, height: 1000 });
  return (
    <picture className="lp-poster">
      <source media="(max-width: 760px)" srcSet={phone} />
      {/* eslint-disable-next-line jsx-a11y/alt-text -- alt comes from getImageProps */}
      <img {...wide} />
    </picture>
  );
}

export default function Home() {
  return (
    <div className="lp lp-home">
      <section className="lp-hero" aria-labelledby="hero-title">
        <div className="lp-hero-copy">
          <h1 id="hero-title" className="lp-lines">
            <span><span>Paid,</span></span>
            <span><span>or paid back.</span></span>
          </h1>
          <p className="lp-sub">
            Arrhabon keeps a student&apos;s tuition deposit in a Solana program. Visa issued: the university is paid. Visa
            refused, or no answer by the deadline: everyone who paid in gets their money back.
          </p>
          <div className="lp-actions">
            <Link className="lp-btn lp-btn-primary" href="/student">Open a vault</Link>
            {liveVault && <Link className="lp-text-link" href={`/vault/${encodeURIComponent(liveVault.id)}`}>See a live vault</Link>}
          </div>
        </div>
        <Poster />
      </section>

      <section className="lp-problem lp-wrap" aria-labelledby="problem-title">
        <div className="lp-problem-copy lp-rise">
          <h2 id="problem-title">Paid before the visa.</h2>
          <p>
            Universities ask for the deposit first. When the visa is refused, the refund can take months or never come.
            In Korea, about 400 students at one university waited on 2.8bn won in refunds after visa denials.
          </p>
          <p className="lp-figure">
            <b><CountUp value={19608} /></b>
            <span>UK student visas refused in the year to June 2026, with 11,500 more applications withdrawn.</span>
          </p>
        </div>
        <figure className="lp-rise">
          <ArtSlot
            className="lp-problem-art"
            src="/art/problem.webp"
            alt="A student sits on a suitcase in front of a shut stone gate. Through a crack in the doors, the gold coin is out of reach. A torn letter lies on the road."
            ratio="16 / 9"
            sizes="(max-width: 760px) 100vw, 58vw"
          />
          <figcaption>Korea Herald, 7 Oct 2026; The PIE News, 5 Oct 2026.</figcaption>
        </figure>
      </section>

      <section className="lp-stairs lp-wrap" id="how-it-works" aria-labelledby="how-title">
        <ol>
          {STAIRS.map((s, i) => (
            <li key={s.head} className="lp-rise">
              {i === 0 ? <h2 id="how-title">{s.head}</h2> : <h3>{s.head}</h3>}
              <p>{s.text}</p>
            </li>
          ))}
        </ol>
        <ArtSlot
          className="lp-strip lp-rise"
          src="/art/how-it-works.webp"
          alt="Five scenes: the student at the bell jar, three cottages sending coins into it, a sealed letter arriving, the coin rolling through the gate, and coins travelling back to the cottages."
          ratio="9 / 2"
          sizes="100vw"
          drawing="steps"
        />
      </section>

      <section className="lp-board" id="where-it-can-go" aria-labelledby="board-title">
        <div className="lp-board-intro lp-wrap">
          <h2 id="board-title" className="lp-rise">Two exits. Nobody can add a third.</h2>
          <p className="lp-rise">
            The program has no instruction that sends the money anywhere else, including to the people who run it.
            Choose a line to read it.
          </p>
        </div>
        <DeparturesBoard />
      </section>

      <section className="lp-plate lp-wrap" aria-labelledby="plate-title">
        <VaultPlate />
      </section>

      <section className="lp-qa" id="questions" aria-labelledby="qa-title">
        <div className="lp-qa-lead"><h2 id="qa-title">Fair questions.</h2></div>
        {QUESTIONS.map((x) => (
          <div key={x.q} className="lp-qa-band">
            <h3>{x.q}</h3>
            <p>{x.a}</p>
          </div>
        ))}
      </section>

      <Link className="lp-go" href="/student" aria-label="Open a vault">
        <span className="lp-go-top">
          <span>Open a vault on devnet with test money. Name the university and the deadline, then send the link to your family.</span>
          <svg viewBox="0 0 48 48" aria-hidden="true"><path d="M12 36 36 12M16 12h20v20" fill="none" stroke="currentColor" strokeWidth="4" /></svg>
        </span>
        <span className="lp-go-word">Open</span>
      </Link>
    </div>
  );
}
