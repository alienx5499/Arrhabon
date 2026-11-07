"use client";
// The landing's annotated vault page: a sample panel at true size, each note level with the part it explains.
import { useEffect, useRef } from "react";
import "./vault-plate.css";

type Note = { part: string; head: string; text: string };

const LEFT: Note[] = [
  { part: "raised", head: "Raised so far", text: "The total in the vault, against the deposit the university asked for." },
  { part: "deadline", head: "The deadline", text: "No decision by then, and the refunds start on their own." },
  { part: "add", head: "Add to this deposit", text: "Log in with an email; the app makes a wallet, no seed phrase." },
];
const RIGHT: Note[] = [
  { part: "paid", head: "Who has paid", text: "Every funder by name, with their share. A refund pays each one back in full." },
  { part: "proof", head: "Proof for the university", text: "The vault account opens in the Solana explorer: funds held, rule in view." },
  { part: "attestor", head: "The one trusted party", text: "The attestor says issued or refused. In this demo it is a labelled test key." },
];
const SAMPLE_FUNDERS = [
  ["Asha (mother)", "400.00", "66.7%"],
  ["Vikram (uncle)", "150.00", "25.0%"],
  ["Rahul (student)", "50.00", "8.3%"],
];
const GAP = 24;
const STACKED = "(max-width: 1080px)";

const Notes = ({ side, notes }: { side: "l" | "r"; notes: Note[] }) => (
  <div className={`plate-notes plate-notes-${side}`}>
    {notes.map((n) => (
      <div key={n.part} className="plate-note" data-for={n.part}>
        <h3>{n.head}</h3>
        <p>{n.text}</p>
      </div>
    ))}
  </div>
);

export function VaultPlate() {
  const root = useRef<HTMLDivElement>(null);
  // Each note sits level with its part; a note that would overlap the one above is pushed down.
  useEffect(() => {
    const el = root.current;
    if (!el) return;
    const place = () =>
      el.querySelectorAll<HTMLElement>(".plate-notes").forEach((col) => {
        const notes = col.querySelectorAll<HTMLElement>(".plate-note");
        if (matchMedia(STACKED).matches) {
          notes.forEach((n) => n.style.removeProperty("top"));
          col.style.removeProperty("min-height");
          return;
        }
        const top0 = col.getBoundingClientRect().top;
        const head = el.querySelector("h2")!.getBoundingClientRect().bottom - top0 + 40;
        let floor = col.classList.contains("plate-notes-l") ? head : 0;
        notes.forEach((n) => {
          const part = el.querySelector(`[data-part="${n.dataset.for}"]`)!;
          const t = Math.max(part.getBoundingClientRect().top - top0, floor);
          n.style.top = `${t}px`;
          floor = t + n.offsetHeight + GAP;
        });
        col.style.minHeight = `${floor}px`;
      });
    place();
    document.fonts?.ready.then(place);
    addEventListener("resize", place);
    return () => removeEventListener("resize", place);
  }, []);

  return (
    <div className="plate" ref={root}>
      <h2 id="plate-title">One page per deposit.</h2>
      <Notes side="l" notes={LEFT} />
      <figure className="plate-panel">
        {/* A picture of the product: sample figures, nothing here is live or clickable. */}
        <div className="vp" aria-hidden="true">
          <div className="vp-title"><b>Deposit for Demo University</b><span className="tag tag-open">Open</span></div>
          <div data-part="raised">
            <p className="vp-raised">600.00 USDC</p>
            <p className="vp-of">raised of 1,000.00 USDC</p>
            <div className="vp-meter"><i /></div>
          </div>
          <dl data-part="deadline">
            <div><dt>Deadline</dt><dd>15 Dec 2026, 18:00</dd></div>
            <div><dt>Time left</dt><dd>42 days</dd></div>
          </dl>
          <div className="vp-add" data-part="add">
            <b>Add to this deposit</b>
            <div className="vp-row"><span className="vp-input">100.00</span><span className="vp-pay">Pay</span></div>
            <small>At least 1 USDC. Test money on devnet.</small>
          </div>
          <table data-part="paid">
            <thead><tr><th>Who has paid</th><th className="r">Paid</th><th className="r">Share</th></tr></thead>
            <tbody>{SAMPLE_FUNDERS.map(([who, paid, share]) => <tr key={who}><td>{who}</td><td className="r">{paid}</td><td className="r">{share}</td></tr>)}</tbody>
          </table>
          <p className="vp-acct" data-part="proof"><span>Vault account</span><u>6p3Y…j8JE</u></p>
          <p className="vp-acct" data-part="attestor"><span>Visa attestor</span><span>Demo attestor</span></p>
        </div>
        <figcaption>An example vault with sample names and amounts.</figcaption>
      </figure>
      <Notes side="r" notes={RIGHT} />
    </div>
  );
}
