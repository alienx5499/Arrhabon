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
};

/** House illustration, or a quiet drawn placeholder while the file is not in public/ yet (checked at render). */
export function ArtSlot({ src, alt, ratio, sizes, priority, className = "" }: Props) {
  const present = existsSync(join(process.cwd(), "public", src));
  return (
    <div className={`art-slot ${className}`} style={{ aspectRatio: ratio }}>
      {present ? (
        <Image src={src} alt={alt} fill sizes={sizes} quality={90} priority={priority} />
      ) : (
        <div className="art-slot-empty" {...(alt ? { role: "img", "aria-label": alt } : { "aria-hidden": true })}>
          <span className="art-slot-jar" />
        </div>
      )}
    </div>
  );
}
