import { describe, expect, it } from "vitest";
import { httpUrl, parseShowcase, showcaseNote } from "./config";

const ID = "3ipoFfDp56kURD4cYvjhynFz1GfNj94gcyWk7BddTEfd";

describe("showcaseNote", () => {
  it("names the outcome from the label", () => {
    expect(showcaseNote("Released")).toBe("Visa issued, deposit paid to the university.");
    expect(showcaseNote("Fully funded, released")).toBe("Visa issued, deposit paid to the university.");
    expect(showcaseNote("Refunded")).toBe("Visa refused, every funder repaid.");
    expect(showcaseNote("Open, still raising")).toBe("Every payment, the decision and where the money went.");
  });
});

describe("parseShowcase", () => {
  it("is empty when unset", () => {
    expect(parseShowcase(undefined)).toEqual([]);
    expect(parseShowcase("  ")).toEqual([]);
  });
  it("reads labels and ids", () => {
    expect(parseShowcase(JSON.stringify([{ label: " Released ", id: ID }]))).toEqual([{ label: "Released", id: ID }]);
  });
  it("rejects malformed input", () => {
    expect(() => parseShowcase("{")).toThrow();
    expect(() => parseShowcase('{"label":"x"}')).toThrow();
    expect(() => parseShowcase(JSON.stringify([{ label: "x", id: "../student" }]))).toThrow();
    expect(() => parseShowcase(JSON.stringify([{ label: "", id: ID }]))).toThrow();
  });
});

describe("httpUrl", () => {
  it("allows unset and http(s)", () => {
    expect(httpUrl(undefined, "X")).toBe("");
    expect(httpUrl(" https://example.com ", "X")).toBe("https://example.com");
  });
  it("rejects other schemes", () => {
    expect(() => httpUrl("javascript:alert(1)", "X")).toThrow();
  });
});
