import type { Metadata } from "next";
import Link from "next/link";
import { ArtSlot } from "@/components/ArtSlot";
import "./landing.css";

export const metadata: Metadata = { title: "Page not found" };

export default function NotFound() {
  return (
    <section className="lp lp-lost" aria-labelledby="lost-title">
      <h1 id="lost-title">This page is off the map.</h1>
      <p className="lp-lead">The link may be old or mistyped. The deposits are where they were.</p>
      <div className="lp-actions">
        <Link className="lp-btn lp-btn-primary" href="/">Back to the front page</Link>
        <Link className="lp-btn lp-btn-quiet" href="/student">Open a vault</Link>
      </div>
      <ArtSlot
        className="lp-lost-art"
        src="/art/not-found.webp"
        alt=""
        ratio="3 / 1"
        sizes="(max-width: 1240px) 100vw, 1200px"
      />
    </section>
  );
}
