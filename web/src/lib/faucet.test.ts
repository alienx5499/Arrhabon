import { describe, expect, it } from "vitest";
import { ApiError } from "./api";
import { faucetMessage, isSolanaAddress } from "./faucet";

const grant = (lamports: number, usdc_micro: string) => ({ demo: true as const, notice: "", signature: "s", lamports, usdc_micro });

describe("faucetMessage", () => {
  it("names what was sent", () => {
    expect(faucetMessage(grant(50_000_000, "1000000000"))).toBe("Sent 0.05 test SOL and 1,000 test USDC to your wallet.");
    expect(faucetMessage(grant(0, "600000000"))).toBe("Sent 600 test USDC to your wallet.");
    expect(faucetMessage(grant(20_000_000, "0"))).toBe("Sent 0.02 test SOL to your wallet.");
  });
  it("explains each refusal in plain words", () => {
    expect(faucetMessage(new ApiError("x", "already_funded", 409))).toMatch(/already has/);
    expect(faucetMessage(new ApiError("x", "faucet_cooldown", 429))).toMatch(/few minutes/);
    expect(faucetMessage(new ApiError("x", "faucet_disabled", 403))).toMatch(/off/);
    expect(faucetMessage(new ApiError("x", "network", 0))).toMatch(/could not send/);
  });
});

describe("isSolanaAddress", () => {
  it("accepts a valid 32-byte base58 key", () => {
    expect(isSolanaAddress("1".repeat(32))).toBe(true);
    expect(isSolanaAddress("A".repeat(44))).toBe(true);
    expect(isSolanaAddress(`  ${"A".repeat(44)}  `)).toBe(true);
  });
  it("rejects empty, too short, too long, and strings with 0, O, I or l", () => {
    expect(isSolanaAddress("")).toBe(false);
    expect(isSolanaAddress("   ")).toBe(false);
    expect(isSolanaAddress("A".repeat(31))).toBe(false);
    expect(isSolanaAddress("A".repeat(45))).toBe(false);
    expect(isSolanaAddress(`0${"A".repeat(31)}`)).toBe(false);
    expect(isSolanaAddress(`O${"A".repeat(31)}`)).toBe(false);
    expect(isSolanaAddress(`I${"A".repeat(31)}`)).toBe(false);
    expect(isSolanaAddress(`l${"A".repeat(31)}`)).toBe(false);
  });
});
