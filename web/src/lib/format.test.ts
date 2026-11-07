import { describe, expect, it } from "vitest";
import { formatUsdc, parseUsdc } from "./usdc";
import { formatShortDeadline, formatTimeLeft } from "./countdown";
import { nameParties, parseParties } from "./config";
import { b64ToBytes, bytesToB64 } from "./api";

describe("usdc", () => {
  it("formats micro-units with 6-decimal precision", () => {
    expect(formatUsdc(0)).toBe("0.00");
    expect(formatUsdc(1_000_000_000)).toBe("1,000.00");
    expect(formatUsdc(1_500_000)).toBe("1.50");
    expect(formatUsdc(1)).toBe("0.000001");
    expect(formatUsdc(4_729_734)).toBe("4.729734");
    expect(formatUsdc("18446744073709551615")).toBe("18,446,744,073,709.551615");
  });
  it("parses decimal text to micro-units without floats", () => {
    expect(parseUsdc("1000")).toBe(1_000_000_000n);
    expect(parseUsdc("0.1")).toBe(100_000n);
    expect(parseUsdc("1,000.000001")).toBe(1_000_000_001n);
    expect(parseUsdc("1.0000001")).toBeNull();
    expect(parseUsdc("-1")).toBeNull();
    expect(parseUsdc("abc")).toBeNull();
  });
});

describe("countdown", () => {
  it("formats remaining time", () => {
    expect(formatTimeLeft(0)).toBe("deadline passed");
    expect(formatTimeLeft(-5)).toBe("deadline passed");
    expect(formatTimeLeft(30)).toBe("under a minute");
    expect(formatTimeLeft(65)).toBe("1 minute");
    expect(formatTimeLeft(8 * 60 + 59)).toBe("8 minutes");
    expect(formatTimeLeft(3600 + 120)).toBe("1 hour 2 minutes");
    expect(formatTimeLeft(4 * 86400 + 6 * 3600 + 4 * 60)).toBe("4 days 6 hours");
    expect(formatTimeLeft(86400 + 59)).toBe("1 day");
  });
  it("writes a compact deadline with the zone's short name", () => {
    const t = Date.UTC(2026, 10, 12, 12, 30) / 1000;
    expect(formatShortDeadline(t, "Asia/Kolkata")).toBe("12 Nov, 18:00 IST");
    expect(formatShortDeadline(t, "UTC")).toBe("12 Nov, 12:30 UTC");
    expect(formatShortDeadline(t, "Europe/London")).toBe("12 Nov, 12:30 GMT");
    // No English locale names Kathmandu's zone, so the offset stands in.
    expect(formatShortDeadline(t, "Asia/Kathmandu")).toBe("12 Nov, 18:15 GMT+5:45");
  });
});

describe("config", () => {
  it("parses Name=address lists", () => {
    expect(parseParties(" Uni A=Abc , Xyz,")).toEqual([
      { name: "Uni A", address: "Abc" },
      { name: "Xyz", address: "Xyz" },
    ]);
    expect(parseParties(undefined)).toEqual([]);
  });
  it("names registry addresses from overrides, else shows the address", () => {
    expect(nameParties(["Abc", "Def"], [{ name: "Uni A", address: "Abc" }, { name: "Gone", address: "Zzz" }])).toEqual([
      { name: "Uni A", address: "Abc" },
      { name: "Def", address: "Def" },
    ]);
  });
});

describe("base64", () => {
  it("round-trips bytes", () => {
    const bytes = new Uint8Array([0, 1, 254, 255, 128]);
    expect(Array.from(b64ToBytes(bytesToB64(bytes)))).toEqual(Array.from(bytes));
  });
});
