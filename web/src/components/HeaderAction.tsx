"use client";
import Link from "next/link";
import { usePathname } from "next/navigation";

/** The header's "Open a vault": gold on the home page only, so every other view keeps one gold button of its own. */
export function HeaderAction() {
  const home = usePathname() === "/";
  return <Link href="/student" className={home ? "btn" : "btn-secondary"}>Open a vault</Link>;
}
