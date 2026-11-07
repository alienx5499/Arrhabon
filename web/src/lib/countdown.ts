/** "2d 03:04:05" style countdown; "deadline passed" at or below zero. */
export function formatCountdown(secondsLeft: number): string {
  if (secondsLeft <= 0) return "deadline passed";
  const s = Math.floor(secondsLeft);
  const days = Math.floor(s / 86400);
  const hms = [Math.floor((s % 86400) / 3600), Math.floor((s % 3600) / 60), s % 60]
    .map((n) => String(n).padStart(2, "0"))
    .join(":");
  return days > 0 ? `${days}d ${hms}` : hms;
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
