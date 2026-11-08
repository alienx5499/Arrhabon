// Aurora price-check inputs: the API's chain ids with readable names, slippage typed as a percent.
import type { SwapStatus } from "./api";
import { formatUsdc, parseUsdc } from "./usdc";

export const CHAINS = [
  { id: "base", name: "Base" },
  { id: "arb", name: "Arbitrum" },
  { id: "eth", name: "Ethereum" },
];

interface NativeUsdcConfig {
  contract: string;
  chainId: number;
}

const NATIVE_USDC: Record<string, NativeUsdcConfig> = {
  base: { contract: "0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913", chainId: 8453 },
  arb: { contract: "0xaf88d065e77c8cC2239327C5EDb3A432268e5831", chainId: 42161 },
  arbitrum: { contract: "0xaf88d065e77c8cC2239327C5EDb3A432268e5831", chainId: 42161 },
  eth: { contract: "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48", chainId: 1 },
  ethereum: { contract: "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48", chainId: 1 },
};

/** EIP-681 token transfer URI for wallets to prefill recipient and micro-units; plain address on fallback. */
export function paymentUri(chain: string, depositAddress: string, amountText: string): string {
  const conf = NATIVE_USDC[chain.trim().toLowerCase()];
  if (!conf) return depositAddress;
  const micro = parseUsdc(amountText);
  if (micro === null || micro <= 0n) return depositAddress;
  return `ethereum:${conf.contract}@${conf.chainId}/transfer?address=${depositAddress}&uint256=${micro.toString()}`;
}

/** "1" or "0.5" percent -> basis points; null outside 0-100 or with more than two decimals. */
export function percentToBps(text: string): number | null {
  const m = /^(\d{1,3})(?:\.(\d{1,2}))?$/.exec(text.trim());
  if (!m) return null;
  const bps = Number(m[1]) * 100 + Number((m[2] ?? "").padEnd(2, "0"));
  return bps <= 10_000 ? bps : null;
}

/** Whether the swap has reached a terminal state and polling should stop. */
export function isFinalStatus(status: SwapStatus | string): boolean {
  return status === "success" || status === "refunded" || status === "failed";
}

/** Plain sentence explaining the current swap status. */
export function swapStatusMessage(status: SwapStatus | string): string {
  switch (status) {
    case "pending":
      return "Waiting for deposit and swap to complete.";
    case "success":
      return "Swap complete. USDC delivered to your Solana address on mainnet.";
    case "refunded":
      return "Swap refunded. Funds returned to your origin address.";
    case "failed":
      return "Swap failed.";
    default:
      return "Waiting for deposit and swap to complete.";
  }
}

/** Plain sentence explaining a mirror failure. */
export function mirrorErrorMessage(code: string): string {
  switch (code) {
    case "swap_pending":
      return "Swap is still pending on mainnet. Please wait for confirmation.";
    case "swap_not_settled":
      return "Swap did not settle on mainnet (refunded or failed).";
    case "unknown_deposit":
      return "Deposit address was not recognized for this session.";
    case "already_mirrored":
      return "This swap was already mirrored to devnet test USDC.";
    case "mirror_disabled":
      return "Devnet mirror is disabled.";
    default:
      return "Could not mirror swap to devnet test USDC.";
  }
}

/** Formats the success message for a devnet mirror grant. */
export function mirrorSuccessMessage(usdcMicro: string | number | bigint): string {
  const n = formatUsdc(usdcMicro);
  return `Swap complete. ${n} test USDC mirrored to your devnet wallet for this vault (devnet test money; the swap itself was real on mainnet).`;
}
