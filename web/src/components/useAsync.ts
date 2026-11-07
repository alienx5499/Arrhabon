"use client";
// Loads data on mount and on demand; keeps the last error for display. Responses from a superseded
// request (changed input, reload, unmount) are ignored so a slow old reply never overwrites a newer one.
import { useCallback, useEffect, useState } from "react";

/** `initial` is server-fetched data shown until the first client load settles. */
export function useAsync<T>(load: (() => Promise<T>) | null, initial: T | null = null) {
  const [data, setData] = useState<T | null>(initial);
  const [error, setError] = useState<unknown>(null);
  const [updatedAt, setUpdatedAt] = useState<number | null>(null);
  const [tick, setTick] = useState(0);
  // The request that last settled; loading is derived so the effect never sets state synchronously.
  const [settled, setSettled] = useState<{ load: typeof load; tick: number } | null>(null);
  const reload = useCallback(() => setTick((t) => t + 1), []);

  useEffect(() => {
    if (!load) return;
    let ignore = false;
    load()
      .then(
        (d) => !ignore && (setData(d), setError(null), setUpdatedAt(Date.now())),
        (e) => !ignore && setError(e),
      )
      .finally(() => !ignore && setSettled({ load, tick }));
    return () => {
      ignore = true;
    };
  }, [load, tick]);
  const loading = load !== null && (settled?.load !== load || settled.tick !== tick);
  return { data, error, loading, reload, updatedAt };
}
