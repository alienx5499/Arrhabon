import { existsSync } from "node:fs";
import { join } from "node:path";
import Image from "next/image";
import "./art-slot.css";

type Props = {
  /** Path under public/, e.g. "/art/hero.webp". */
  src: string;
  alt: string;
  /** CSS aspect-ratio, e.g. "16 / 9". */
  ratio: string;
  sizes: string;
  priority?: boolean;
  className?: string;
  /** Placeholder drawing until the file lands: a jar, a jar before a gateway, or five steps. */
  drawing?: "jar" | "gateway" | "steps";
};

function Drawing({ drawing }: { drawing: NonNullable<Props["drawing"]> }) {
  if (drawing === "steps")
    return (
      <span className="art-slot-steps">
        {[1, 2, 3, 4, 5].map((n) => <span key={n} className="art-slot-step" />)}
      </span>
    );
  return (
    <span className="art-slot-scene">
      <span className="art-slot-jar" />
      {drawing === "gateway" && <span className="art-slot-gate" />}
    </span>
  );
}

/** House illustration, or a quiet drawn placeholder while the file is not in public/ yet (checked at render). */
export function ArtSlot({ src, alt, ratio, sizes, priority, className = "", drawing = "jar" }: Props) {
  const present = existsSync(join(process.cwd(), "public", src));
  return (
    <div className={`art-slot ${className}`} style={{ aspectRatio: ratio }}>
      {present ? (
        <Image src={src} alt={alt} fill sizes={sizes} quality={90} priority={priority} />
      ) : (
        <div className="art-slot-empty" {...(alt ? { role: "img", "aria-label": alt } : { "aria-hidden": true })}>
          <Drawing drawing={drawing} />
        </div>
      )}
    </div>
  );
}
