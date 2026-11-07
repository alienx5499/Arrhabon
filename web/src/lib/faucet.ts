import { ApiError, type FaucetGrant } from "./api";

const BASE58_ADDRESS = /^[1-9A-HJ-NP-Za-km-z]{32,44}$/;

/** Pure check for a base58 Solana address: trimmed, 32 to 44 chars, no 0, O, I, l. */
export function isSolanaAddress(s: string): boolean {
  return typeof s === "string" && BASE58_ADDRESS.test(s.trim());
}

/** One line for the header after a faucet claim: what arrived, or why nothing did. */
export function faucetMessage(result: FaucetGrant | unknown): string {
  if (result instanceof ApiError) {
    switch (result.code) {
      case "already_funded":
        return "This wallet already has test SOL and USDC.";
      case "faucet_cooldown":
        return "Just funded. Try again in a few minutes.";
      case "faucet_busy":
        return "The test faucet is busy. Try again later, or get devnet SOL at faucet.solana.com.";
      case "faucet_disabled":
        return "The test faucet is off on this server.";
      default:
        return "The faucet could not send right now. Try again shortly.";
    }
  }
  const g = result as FaucetGrant;
  const parts: string[] = [];
  if (g.lamports > 0) parts.push(`${(g.lamports / 1e9).toFixed(2)} test SOL`);
  const usdc = Number(BigInt(g.usdc_micro) / 1_000_000n);
  if (usdc > 0) parts.push(`${usdc.toLocaleString("en-US")} test USDC`);
  return `Sent ${parts.join(" and ")} to your wallet.`;
}
