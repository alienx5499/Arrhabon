// The ways out for this deposit, each with its picture and this vault's own amounts.
import Image from "next/image";
import type { ExitRule } from "@/lib/outcome";

const ART: Record<ExitRule["key"], { src: string; alt: string }> = {
  issued: { src: "/art/released.webp", alt: "The student walks through the open stone gate; the gold coin rolls ahead of them." },
  refused: { src: "/art/refunded.webp", alt: "The gate is shut; coins travel back along a dashed path to three lit cottages." },
};

export function VaultExits({ rules }: { rules: ExitRule[] }) {
  if (rules.length === 0) return null;
  return (
    <section className="vault-exits" aria-labelledby="exits-head">
      <h2 id="exits-head">{rules.length > 1 ? "Two ways out for this deposit" : "Where the money goes"}</h2>
      <div className="vault-exits-pair">
        {rules.map((r) => (
          <figure key={r.key} className={`vault-exit-${r.key}`}>
            <div className="vault-exit-art">
              <Image src={ART[r.key].src} alt={ART[r.key].alt} fill sizes="(max-width: 900px) 100vw, 400px" quality={90} />
            </div>
            <figcaption><b>{r.head}</b>{r.text}</figcaption>
          </figure>
        ))}
      </div>
    </section>
  );
}
