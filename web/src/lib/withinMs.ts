/** The promise's value, or null if it rejects or takes longer than `ms`. */
export function withinMs<T>(p: Promise<T>, ms: number): Promise<T | null> {
  let timer: ReturnType<typeof setTimeout> | undefined;
  const late = new Promise<null>((resolve) => (timer = setTimeout(() => resolve(null), ms)));
  return Promise.race([p.catch(() => null), late]).finally(() => clearTimeout(timer));
}
