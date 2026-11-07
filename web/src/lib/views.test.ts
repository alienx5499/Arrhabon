import { describe, expect, it } from "vitest";
import { ApiError, type Vault } from "./api";
import { displayName, nameParties, parseNameMap } from "./config";
import { formatDeadline } from "./countdown";
import { OUTCOME_TEXT, funderTag, outcomeOf, universityTotals } from "./outcome";
import { classifyProblem } from "./problem";
import { percentToBps } from "./quote";
import { withinMs } from "./withinMs";

const LONG = "6p3YF6s88hUk6t2mtNQ792gDBkMUxQxYRTCYafqGj8JE";
const c = (amount: number, refunded = false) => ({ funder: `f${amount}`, amount, refunded });
const vault = (status: Vault["status"], contributors: Vault["contributors"]): Vault => ({
  vault: "v",
  student: "s",
  university: "u",
  attestor: "a",
  status,
  target: 100,
  total_contributed: contributors.reduce((s, x) => s + x.amount, 0),
  deadline: 0,
  seconds_left: 0,
  contributors,
});

describe("outcome", () => {
  it("splits closed vaults by how they ended", () => {
    expect(outcomeOf("open", [])).toBe("open");
    expect(outcomeOf("refunding", [c(1)])).toBe("refunding");
    expect(outcomeOf("closed", [])).toBe("empty");
    expect(outcomeOf("closed", [c(1, true), c(2, true)])).toBe("refunded");
    expect(outcomeOf("closed", [c(1), c(2)])).toBe("paid");
  });
  it("says what happens next for refunds", () => {
    expect(OUTCOME_TEXT.refunding.body).toContain("nobody can redirect this money");
  });
  it("tags each funder", () => {
    expect(funderTag("refunding", false)).toBe("refund pending");
    expect(funderTag("refunding", true)).toBe("refunded");
    expect(funderTag("refunded", true)).toBe("refunded");
    expect(funderTag("open", false)).toBe("held");
    expect(funderTag("paid", false)).toBe("paid to the university");
  });
  it("totals a university's vaults", () => {
    const t = universityTotals([
      vault("open", [c(10)]),
      vault("refunding", [c(5, true), c(7)]),
      vault("released", [c(100)]),
      vault("closed", [c(50)]),
      vault("closed", [c(3, true)]),
      vault("closed", []),
    ]);
    expect(t).toEqual({ held: 17n, released: 150n, refunded: 8n, count: 6 });
  });
});

describe("problem", () => {
  it("treats an unreachable or starting API as waking up, with the raw text as detail", () => {
    const p = classifyProblem(new ApiError("API unreachable at http://x", "network", 0));
    expect(p.retry).toBe(true);
    expect(p.message).toContain("waking up");
    expect(p.detail).toContain("API unreachable");
    expect(classifyProblem(new ApiError("bad gateway", "http", 502)).retry).toBe(true);
  });
  it("hides API detail for other failures", () => {
    expect(classifyProblem(new ApiError("no such vault", "vault_not_found", 404))).toMatchObject({ retry: false, message: expect.stringContaining("Not found") });
    expect(classifyProblem(new ApiError("amount too small", "invalid", 400)).detail).toContain("amount too small");
  });
  it("retries RPC failures and passes app errors through", () => {
    expect(classifyProblem(new Error("RPC HTTP 429"))).toMatchObject({ retry: true, detail: "RPC HTTP 429" });
    expect(classifyProblem(new TypeError("Failed to fetch")).retry).toBe(true);
    expect(classifyProblem(new Error("Cancelled: nothing was signed."))).toEqual({ message: "Cancelled: nothing was signed.", detail: null, retry: false });
  });
});

describe("names", () => {
  it("parses the JSON name map, ignoring junk", () => {
    expect(parseNameMap('{"Abc":"Uni A","Bad":3}')).toEqual({ Abc: "Uni A" });
    expect(parseNameMap("not json")).toEqual({});
    expect(parseNameMap("[1]")).toEqual({});
    expect(parseNameMap(undefined)).toEqual({});
  });
  it("looks up the map, then the Name=address list, else shortens the address", () => {
    const parties = [{ name: "From list", address: "Abc" }];
    expect(displayName("Abc", { Abc: "From map" }, parties)).toBe("From map");
    expect(displayName("Abc", {}, parties)).toBe("From list");
    expect(displayName(LONG, {}, [])).toBe("6p3Y…j8JE");
    expect(nameParties([LONG], [], { [LONG]: "Uni B" })).toEqual([{ name: "Uni B", address: LONG }]);
  });
});

describe("deadline", () => {
  it("formats in the given zone and names the zone", () => {
    expect(formatDeadline(0, "en-GB", "UTC")).toBe("1 Jan 1970, 00:00 UTC");
    expect(formatDeadline(0, "en-GB", "Asia/Kolkata")).toBe("1 Jan 1970, 05:30 GMT+5:30");
  });
});

describe("withinMs", () => {
  it("returns the value, or null on rejection or timeout", async () => {
    expect(await withinMs(Promise.resolve(1), 50)).toBe(1);
    expect(await withinMs(Promise.reject(new Error("x")), 50)).toBeNull();
    expect(await withinMs(new Promise(() => {}), 10)).toBeNull();
  });
});

describe("quote", () => {
  it("reads slippage percent as basis points", () => {
    expect(percentToBps("1")).toBe(100);
    expect(percentToBps("0.5")).toBe(50);
    expect(percentToBps("100")).toBe(10_000);
    expect(percentToBps("100.01")).toBeNull();
    expect(percentToBps("1.234")).toBeNull();
    expect(percentToBps("-1")).toBeNull();
  });
});
