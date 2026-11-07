import { describe, expect, it } from "vitest";
import { ApiError, type Vault } from "./api";
import { displayName, nameParties, parseNameMap } from "./config";
import { formatDeadline } from "./countdown";
import { OUTCOME_TEXT, TAG_OF, exitRules, funderTag, outcomeOf, shareOf, tagLabel, universityTotals, vaultFacts } from "./outcome";
import { classifyProblem } from "./problem";
import { percentToBps } from "./quote";
import { withinMs } from "./withinMs";

const LONG = "3ipoFfDp56kURD4cYvjhynFz1GfNj94gcyWk7BddTEfd";
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
  it("tags a vault by how it ended", () => {
    const tag = (v: Vault) => TAG_OF[outcomeOf(v.status, v.contributors)];
    expect(tag(vault("open", []))).toBe("open");
    expect(tag(vault("released", [c(100)]))).toBe("released");
    expect(tag(vault("closed", [c(100)]))).toBe("released");
    expect(tag(vault("refunding", [c(5)]))).toBe("refunding");
    expect(tag(vault("closed", [c(5, true)]))).toBe("refunded");
    expect(tag(vault("closed", []))).toBe("closed");
    expect(tagLabel("refunding")).toBe("Refunding");
    expect(tagLabel("refunded")).toBe("Refunded");
  });
  it("gives each funder a whole-percent share", () => {
    expect(shareOf(400, 600)).toBe("67%");
    expect(shareOf(50, 600)).toBe("8%");
    expect(shareOf(1, 1000)).toBe("<1%");
    expect(shareOf(0, 0)).toBe("0%");
  });
  it("shows time left only while open, and when a closed vault closed", () => {
    const fmt = (t: number) => `t${t}`;
    const labels = (v: Vault, closedAt?: number | null) => vaultFacts(v, outcomeOf(v.status, v.contributors), closedAt, fmt).map((f) => f.label);
    expect(labels(vault("open", [c(1)]))).toEqual(["Time left", "Deadline", "Funders"]);
    expect(labels(vault("refunding", [c(1)]))).toEqual(["Deadline", "Funders"]);
    expect(labels(vault("closed", [c(1)]), null)).toEqual(["Closed on", "Deadline", "Funders"]);
    expect(vaultFacts(vault("closed", [c(1)]), "paid", null, fmt)[0]).toEqual({ label: "Closed on", text: "Not known" });
    expect(vaultFacts(vault("closed", [c(1)]), "paid", 7, fmt)[0]).toEqual({ label: "Closed on", text: "t7" });
    expect(vaultFacts(vault("closed", [c(1)]), "paid", undefined, fmt)[0]).toEqual({ label: "Closed on", text: null });
  });
  const usdc = (m: number) => String(m);
  const rulesOf = (v: Vault) => exitRules(v, outcomeOf(v.status, v.contributors), "Uni", usdc);
  it("writes the exits with the vault's numbers", () => {
    expect(rulesOf(vault("open", [c(100), c(20)])).map((r) => r.text)).toEqual([
      "120 USDC goes to Uni in one payment.",
      "Each of the 2 funders below gets back their own amount, 120 USDC in all.",
    ]);
    expect(rulesOf(vault("open", [c(30)])).map((r) => r.text)).toEqual([
      "Once 100 USDC is raised, the whole deposit goes to Uni in one payment.",
      "The funder below gets back their own amount, 30 USDC in all.",
    ]);
    expect(rulesOf(vault("open", []))[1].text).toBe("Anyone who pays in gets back their own amount.");
  });
  it("shows both exits while open and only the one taken once finished", () => {
    const keys = (v: Vault) => rulesOf(v).map((r) => r.key);
    expect(keys(vault("open", [c(1)]))).toEqual(["issued", "refused"]);
    expect(keys(vault("released", [c(100)]))).toEqual(["issued"]);
    expect(keys(vault("closed", [c(100)]))).toEqual(["issued"]);
    expect(keys(vault("refunding", [c(5)]))).toEqual(["refused"]);
    expect(keys(vault("closed", [c(5, true)]))).toEqual(["refused"]);
    expect(keys(vault("closed", []))).toEqual([]);
    expect(rulesOf(vault("closed", [c(100)]))[0].text).toBe("The visa was issued. The whole deposit went to Uni.");
    expect(rulesOf(vault("closed", [c(60, true), c(40, true)]))[0].text).toBe(
      "The visa was refused or the deadline passed. Each of the 2 funders got back their own amount, 100 USDC in all.",
    );
  });
});

describe("problem", () => {
  it("treats an unreachable or starting API as starting up, with the raw text as detail", () => {
    const p = classifyProblem(new ApiError("API unreachable at http://x", "network", 0));
    expect(p.retry).toBe(true);
    expect(p.message).toBe("The vault service is starting up. This can take up to a minute.");
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
    expect(displayName(LONG, {}, [])).toBe("3ipo…TEfd");
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
