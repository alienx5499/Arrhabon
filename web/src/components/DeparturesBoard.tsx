"use client";
// The landing's departures board: every place the deposit could go, two real exits and the attempts the program refuses.
import { useId, useState } from "react";
import "./departures-board.css";

type Line = { when: string; who: string; to: string; status: "paid" | "refund" | "none"; detail: string };

const STATUS_TEXT = { paid: "Paid", refund: "Refund", none: "No exit" } as const;

const LINES: Line[] = [
  { when: "Visa issued", who: "The attestor", to: "The university", status: "paid", detail: "The vault must be fully funded and the deadline not yet passed. The whole balance goes to the university in one transaction." },
  { when: "Visa refused", who: "The attestor", to: "Every funder", status: "refund", detail: "The vault switches to refunding. Each funder gets back exactly what they paid, one refund per person, never twice." },
  { when: "No answer by the deadline", who: "Anyone", to: "Every funder", status: "refund", detail: "Anyone can start the expiry and each refund: the crank, a funder, a stranger. If the website disappears, a funder can still call it." },
  { when: "The operator wants the money", who: "No one can", to: "Nowhere", status: "none", detail: "The deposit sits in an account owned by the program, not by a person. No private key can move it, mine included." },
  { when: "Attestor confirms after the deadline", who: "Refused", to: "Nowhere", status: "none", detail: "A late confirmation fails. Once the deadline passes, the only way out is the refund." },
  { when: "The student names a friend as attestor", who: "Refused", to: "Nowhere", status: "none", detail: "Universities and attestors must be on the registry, and student, university and attestor must be three different parties." },
  { when: "A refund asked for twice", who: "Refused", to: "Nowhere", status: "none", detail: "Each payment is marked once it is refunded, so the same money cannot be paid back again." },
];

/** One line open at a time; choosing the open line closes it. */
export function DeparturesBoard() {
  const [open, setOpen] = useState<number | null>(3);
  const id = useId();
  return (
    <div className="board">
      <div className="board-head" aria-hidden="true">
        <span>When</span>
        <span>Who signs</span>
        <span>Money goes to</span>
        <span />
      </div>
      <div className="board-cut" />
      <ul>
        {LINES.map((l, i) => {
          const expanded = open === i;
          return (
            <li key={l.when} className={expanded ? "is-open" : undefined}>
              <button type="button" aria-expanded={expanded} aria-controls={`${id}-${i}`} onClick={() => setOpen(expanded ? null : i)}>
                <span>{l.when}</span>
                <span className="board-who">{l.who}</span>
                <span className="board-to">{l.to}</span>
                <span className={`board-st board-st-${l.status}`}>{STATUS_TEXT[l.status]}</span>
              </button>
              <p id={`${id}-${i}`} className="board-detail" hidden={!expanded}>{l.detail}</p>
            </li>
          );
        })}
      </ul>
      <div className="board-cut" />
    </div>
  );
}
