"use client";
// A figure that rolls up from zero the first time it scrolls into view; the final value is in the page from the start.
import { useEffect, useRef, useState } from "react";

const DURATION_MS = 1400;

export function CountUp({ value }: { value: number }) {
  const ref = useRef<HTMLSpanElement>(null);
  const [shown, setShown] = useState(value);

  useEffect(() => {
    const el = ref.current;
    if (!el || matchMedia("(prefers-reduced-motion: reduce)").matches) return;
    let frame = 0;
    const roll = (start: number) => (now: number) => {
      const t = Math.min((now - start) / DURATION_MS, 1);
      setShown(Math.round(value * (1 - (1 - t) ** 3)));
      if (t < 1) frame = requestAnimationFrame(roll(start));
    };
    const seen = new IntersectionObserver(([entry]) => {
      if (!entry.isIntersecting) return;
      seen.disconnect();
      frame = requestAnimationFrame((now) => roll(now)(now));
    }, { threshold: 0.6 });
    setShown(0);
    seen.observe(el);
    return () => {
      seen.disconnect();
      cancelAnimationFrame(frame);
    };
  }, [value]);

  return (
    <span ref={ref} role="img" aria-label={value.toLocaleString("en-GB")}>
      <span aria-hidden="true">{shown.toLocaleString("en-GB")}</span>
    </span>
  );
}
