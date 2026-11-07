const unit = (n: number, word: string) => `${n} ${word}${n === 1 ? "" : "s"}`;

/** Time left in words, the two largest units: "4 days 6 hours", "8 minutes"; "deadline passed" at or below zero. */
export function formatTimeLeft(secondsLeft: number): string {
  if (secondsLeft <= 0) return "deadline passed";
  const s = Math.floor(secondsLeft);
  const [d, h, m] = [Math.floor(s / 86400), Math.floor((s % 86400) / 3600), Math.floor((s % 3600) / 60)];
  if (d > 0) return h > 0 ? `${unit(d, "day")} ${unit(h, "hour")}` : unit(d, "day");
  if (h > 0) return m > 0 ? `${unit(h, "hour")} ${unit(m, "minute")}` : unit(h, "hour");
  return m > 0 ? unit(m, "minute") : "under a minute";
}

/** The zone's short name ("IST", "BST", "PST"); a "GMT+5:30" style offset only when no locale knows a name. */
function zoneName(date: Date, timeZone?: string): string {
  let offset = "";
  // shortcut: a fixed list of English locales; extend it if a zone shows an offset where people expect a name.
  for (const locale of ["en-US", "en-GB", "en-IN", "en-AU"]) {
    const name = new Intl.DateTimeFormat(locale, { timeZoneName: "short", timeZone }).formatToParts(date).find((p) => p.type === "timeZoneName")?.value ?? "";
    if (name && !/^(GMT|UTC)[+-−]/.test(name)) return name;
    offset ||= name;
  }
  return offset;
}

/** Compact local date and time with the zone, e.g. "12 Nov, 18:00 IST". */
export function formatShortDeadline(unix: number, timeZone?: string): string {
  const date = new Date(unix * 1000);
  const when = new Intl.DateTimeFormat("en-GB", { day: "numeric", month: "short", hour: "2-digit", minute: "2-digit", hourCycle: "h23", timeZone }).format(date);
  return `${when} ${zoneName(date, timeZone)}`;
}

/** Unix seconds as the viewer's local date and time with the time zone, e.g. "4 Nov 2026, 18:30 GMT+5:30". */
export function formatDeadline(unix: number, locale?: string, timeZone?: string): string {
  return new Intl.DateTimeFormat(locale, {
    day: "numeric",
    month: "short",
    year: "numeric",
    hour: "2-digit",
    minute: "2-digit",
    timeZoneName: "short",
    timeZone,
  }).format(new Date(unix * 1000));
}
