import { describe, expect, it } from "vitest";
import {
  isFinalStatus,
  mirrorErrorMessage,
  mirrorSuccessMessage,
  percentToBps,
  swapStatusMessage,
} from "./quote";

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

describe("mirrorErrorMessage", () => {
  it("maps every new error code to a plain sentence", () => {
    expect(mirrorErrorMessage("swap_pending")).toBe(
      "Swap is still pending on mainnet. Please wait for confirmation.",
    );
    expect(mirrorErrorMessage("swap_not_settled")).toBe(
      "Swap did not settle on mainnet (refunded or failed).",
    );
    expect(mirrorErrorMessage("unknown_deposit")).toBe(
      "Deposit address was not recognized for this session.",
    );
    expect(mirrorErrorMessage("already_mirrored")).toBe(
      "This swap was already mirrored to devnet test USDC.",
    );
    expect(mirrorErrorMessage("mirror_disabled")).toBe("Devnet mirror is disabled.");
    expect(mirrorErrorMessage("other")).toBe("Could not mirror swap to devnet test USDC.");
  });
});

describe("mirrorSuccessMessage", () => {
  it("formats micro units correctly", () => {
    expect(mirrorSuccessMessage("100000000")).toBe(
      "Swap complete. 100.00 test USDC mirrored to your devnet wallet for this vault (devnet test money; the swap itself was real on mainnet).",
    );
    expect(mirrorSuccessMessage("5000000")).toBe(
      "Swap complete. 5.00 test USDC mirrored to your devnet wallet for this vault (devnet test money; the swap itself was real on mainnet).",
    );
    expect(mirrorSuccessMessage(25500000)).toBe(
      "Swap complete. 25.50 test USDC mirrored to your devnet wallet for this vault (devnet test money; the swap itself was real on mainnet).",
    );
  });
});

