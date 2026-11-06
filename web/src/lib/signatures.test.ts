import { describe, expect, it, vi } from "vitest";
import { recentSignatures } from "./signatures";

const VAULT = "7EYnhQoR9YM3N7UoaKRoA44Uy8JeaZV3qyouov87awMs";

function stub(body: unknown, ok = true) {
  return vi.fn<typeof fetch>(async () => ({ ok, status: ok ? 200 : 500, json: async () => body }) as Response);
}

describe("recentSignatures", () => {
  it("asks for the newest 10 and maps the rows", async () => {
    const fetchFn = stub({ result: [{ signature: "s2", blockTime: 20, err: null }, { signature: "s1", blockTime: null, err: { x: 1 } }] });
    const rows = await recentSignatures(VAULT, "http://rpc", fetchFn);
    expect(rows).toEqual([
      { signature: "s2", blockTime: 20, failed: false },
      { signature: "s1", blockTime: null, failed: true },
    ]);
    const [url, init] = fetchFn.mock.calls[0];
    expect(url).toBe("http://rpc");
    expect(JSON.parse(String(init?.body))).toMatchObject({ method: "getSignaturesForAddress", params: [VAULT, { limit: 10, commitment: "confirmed" }] });
  });
  it("throws on an RPC error or HTTP failure", async () => {
    await expect(recentSignatures(VAULT, "http://rpc", stub({ error: { message: "bad" } }))).rejects.toThrow(/bad/);
    await expect(recentSignatures(VAULT, "http://rpc", stub(null, false))).rejects.toThrow(/500/);
  });
});
