import Link from "next/link";

const SOURCES = [
  { name: "The Korea Herald", date: "7 Oct 2026", url: "https://www.koreaherald.com/article/10895948" },
  {
    name: "ICEF Monitor",
    date: "10 Sep 2026",
    url: "https://monitor.icef.com/2026/09/korean-government-introduces-new-quality-controls-for-universities-and-agents/",
  },
  {
    name: "The PIE News",
    date: "5 Oct 2026",
    url: "https://thepienews.com/in-focus-how-uk-unis-are-staying-under-the-5-bca-threshold/",
  },
];

// Each figure is reported separately by its source; they are never summed.
const FACTS = [
  { figure: "about 400", text: "international students at Singyeongju University still owed tuition refunds after visa denials", source: "The Korea Herald" },
  { figure: "about 170", text: "international students at Woongji still owed tuition refunds after visa denials", source: "The Korea Herald" },
  { figure: "19,608", text: "student visas refused by the UK in the year to June 2026", source: "The PIE News" },
];

const STEPS = [
  { title: "Create", text: "The student creates a vault naming the university, the visa attestor, a target and a deadline." },
  { title: "Fund", text: "Family members pay in to the vault link the student shares." },
  { title: "Visa issued", text: "The university is paid once the visa is issued." },
  { title: "Denied or expired", text: "If it is denied or nobody confirms by the deadline, every family member who paid in gets their contribution back." },
];

export default function Home() {
  return (
    <div className="flex flex-col gap-8">
      <section className="lead cols" aria-labelledby="lead-headline">
        <article className="flex flex-col gap-4">
          <p className="label label-accent">Lead</p>
          <h1 id="lead-headline" className="headline">A tuition deposit that refunds itself</h1>
          <p className="story">
            In October 2026 The Korea Herald reported that about 400 international students at Singyeongju University
            and about 170 at Woongji were still owed tuition refunds after their visas were denied. Korea&apos;s education
            ministry already requires refunding students whose visa is denied (ICEF Monitor, 10 Sep 2026), and the UK
            refused 19,608 student visas in the year to June 2026 (The PIE News, 5 Oct 2026). The rule exists; the money
            still sits with the university. Arrhabon holds the deposit in a Solana program instead: the university is
            paid once the visa is issued, and if it is denied or nobody confirms by the deadline, every family member
            who paid in gets their contribution back.
          </p>
        </article>
        <aside className="flex flex-col" aria-label="Actions">
          <div className="flex flex-col gap-3 pb-4">
            <h2 className="section-head mb-0">Create a vault</h2>
            <Link href="/student" className="btn self-start">I&apos;m a student: create a vault</Link>
          </div>
          <div className="rule-hair flex flex-col gap-2 pt-4">
            <h2 className="section-head mb-0">Open a vault</h2>
            <p className="compact">
              Funders open the vault link the student shares. Universities open /university/&lt;their address&gt;.
            </p>
          </div>
        </aside>
      </section>

      <section className="rule-section pt-3" aria-label="Figures">
        <dl className="cols">
          {FACTS.map((f) => (
            <div key={f.figure} className="fact">
              <dt className="fact-figure">{f.figure}</dt>
              <dd className="compact">
                {f.text} <span className="label muted whitespace-nowrap">{f.source}</span>
              </dd>
            </div>
          ))}
        </dl>
      </section>

      <section className="rule-section pt-3" aria-labelledby="how-heading">
        <h2 id="how-heading" className="label label-accent mb-3">How it works</h2>
        <ol className="cols">
          {STEPS.map((s, i) => (
            <li key={s.title}>
              <h3 className="section-head mb-1 text-[20px]">
                {i + 1}. {s.title}
              </h3>
              <p className="compact">{s.text}</p>
            </li>
          ))}
        </ol>
      </section>

      <section aria-labelledby="sources-heading" className="rule-section pt-3 text-sm">
        <h2 id="sources-heading" className="label label-accent mb-2">Sources</h2>
        <ul className="listing">
          {SOURCES.map((s) => (
            <li key={s.name} className="listing-row">
              <a className="link field-label" href={s.url} target="_blank" rel="noreferrer">
                {s.name}
              </a>
              <span className="muted">{s.date}</span>
            </li>
          ))}
        </ul>
        <p className="mt-3 muted">Runs on Solana devnet with test USDC. The visa attestor is the one trusted party.</p>
      </section>
    </div>
  );
}
