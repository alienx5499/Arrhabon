import { describe, expect, it } from "vitest";
import { isFinalStatus, percentToBps, swapStatusMessage } from "./quote";

describe("swapStatusMessage", () => {
  it("maps every status value to a sentence", () => {
    expect(swapStatusMessage("pending")).toBe("Waiting for deposit and swap to complete.");
    expect(swapStatusMessage("success")).toBe("Swap complete. USDC delivered to your Solana address on mainnet.");
    expect(swapStatusMessage("refunded")).toBe("Swap refunded. Funds returned to your origin address.");
    expect(swapStatusMessage("failed")).toBe("Swap failed.");
    expect(swapStatusMessage("unknown")).toBe("Waiting for deposit and swap to complete.");
  });
});

describe("isFinalStatus", () => {
  it("flags final states as final", () => {
    expect(isFinalStatus("success")).toBe(true);
    expect(isFinalStatus("refunded")).toBe(true);
    expect(isFinalStatus("failed")).toBe(true);
    expect(isFinalStatus("pending")).toBe(false);
    expect(isFinalStatus("other")).toBe(false);
  });
});

describe("percentToBps", () => {
  it("converts valid percentages to basis points", () => {
    expect(percentToBps("1")).toBe(100);
    expect(percentToBps("0.5")).toBe(50);
    expect(percentToBps("0.05")).toBe(5);
    expect(percentToBps("100")).toBe(10000);
  });
  it("rejects invalid percentages", () => {
    expect(percentToBps("101")).toBeNull();
    expect(percentToBps("0.001")).toBeNull();
    expect(percentToBps("abc")).toBeNull();
  });
});

