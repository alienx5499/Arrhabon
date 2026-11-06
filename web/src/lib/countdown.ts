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
