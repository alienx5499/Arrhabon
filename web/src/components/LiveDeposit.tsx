"use client";
import { useEffect, useRef, useState } from "react";
import { QRCodeSVG } from "qrcode.react";
import { api, ApiError, type MirrorGrant, type Quote, type SwapStatus } from "@/lib/api";
import { formatDeadline } from "@/lib/countdown";
import { isFinalStatus, mirrorErrorMessage, mirrorSuccessMessage, paymentUri, swapStatusMessage } from "@/lib/quote";
import { formatUsdc } from "@/lib/usdc";
import { TxLink, Usdc } from "./ui";

export type LiveDepositProps = {
  quote: Quote;
  chain: string;
  asset: string;
  amount: string;
};

export function LiveDeposit({ quote, chain, asset, amount }: LiveDepositProps) {
  const expiryMs = typeof quote.expires_at === "number" ? quote.expires_at * 1000 : Date.parse(quote.expires_at);
  const [copied, setCopied] = useState(false);
  const [status, setStatus] = useState<SwapStatus | null>("pending");
  const [expired, setExpired] = useState(() => Number.isFinite(expiryMs) && Date.now() >= expiryMs);
  const [mirror, setMirror] = useState<MirrorGrant | null>(null);
  const [mirrorError, setMirrorError] = useState<string | null>(null);
  const mirroredRef = useRef<string | null>(null);

  useEffect(() => {
    if (status !== "success" || mirroredRef.current === quote.deposit_address) return;
    mirroredRef.current = quote.deposit_address;
    api
      .mirror(quote.deposit_address)
      .then((m) => {
        setMirror(m);
        setMirrorError(null);
      })
      .catch((err) => {
        if (err instanceof ApiError) {
          setMirrorError(mirrorErrorMessage(err.code));
        } else {
          setMirrorError("Could not mirror swap to devnet test USDC.");
        }
      });
  }, [status, quote.deposit_address]);

  useEffect(() => {
    const isExpired = () => Number.isFinite(expiryMs) && Date.now() >= expiryMs;
    if (isExpired()) {
      return;
    }

    let active = true;
    let pollTimer: ReturnType<typeof setInterval> | null = null;
    let expiryTimer: ReturnType<typeof setTimeout> | null = null;

    const msLeft = expiryMs - Date.now();
    if (Number.isFinite(msLeft) && msLeft > 0) {
      expiryTimer = setTimeout(() => {
        setExpired(true);
        if (pollTimer) clearInterval(pollTimer);
      }, msLeft);
    }

    async function check() {
      if (!active || isExpired()) return;
      try {
        const res = await api.fundingStatus(quote.deposit_address);
        if (!active) return;
        setStatus(res.status);
        if (isFinalStatus(res.status)) {
          if (pollTimer) clearInterval(pollTimer);
          if (expiryTimer) clearTimeout(expiryTimer);
        }
      } catch {
        // server unreachable or network error: keep polling until expiry
      }
    }

    check();
    pollTimer = setInterval(check, 5000);

    return () => {
      active = false;
      if (pollTimer) clearInterval(pollTimer);
      if (expiryTimer) clearTimeout(expiryTimer);
    };
  }, [expiryMs, quote.deposit_address]);

  function copy() {
    if (typeof navigator !== "undefined" && navigator.clipboard) {
      navigator.clipboard.writeText(quote.deposit_address).then(() => {
        setCopied(true);
        setTimeout(() => setCopied(false), 2000);
      });
    }
  }

  const deadline = typeof quote.expires_at === "number" ? formatDeadline(quote.expires_at) : quote.expires_at;
  const showQr = !expired && !isFinalStatus(status ?? "");

  return (
    <div className="mt-4 flex flex-col gap-4">
      <div className="flex flex-col gap-4 md:flex-row md:items-start md:justify-between">
        <div className="flex flex-1 flex-col gap-2 min-w-0">
          <span className="field-label">Deposit address</span>
          <div className="flex flex-wrap items-center gap-3">
            <code className="font-mono text-sm break-all">{quote.deposit_address}</code>
            <button type="button" className="btn-secondary" onClick={copy}>
              {copied ? "Copied" : "Copy"}
            </button>
          </div>
          <p className="text-sm">
            Send exactly {amount} {asset} on {chain} to this address. Aurora swaps it and delivers USDC on Solana.
          </p>
        </div>

        {showQr && (
          <div className="flex flex-col items-center gap-2 self-start shrink-0">
            <div
              className="p-3"
              style={{
                backgroundColor: "#fffdf6",
                borderRadius: "var(--radius-card)",
                border: "1px solid var(--color-placeholder)",
              }}
            >
              <QRCodeSVG
                value={paymentUri(chain, quote.deposit_address, amount)}
                size={200}
                fgColor="#151d30"
                bgColor="#fffdf6"
                level="M"
              />
            </div>
            <p className="text-sm text-center">
              Scan with your wallet: sends {amount} USDC on {chain}
            </p>
          </div>
        )}
      </div>

      <dl className="listing text-sm">
        <div className="listing-row">
          <dt>At least this arrives</dt>
          <dd><Usdc micro={quote.min_usdc_out} /></dd>
        </div>
        <div className="listing-row">
          <dt>Fee</dt>
          <dd><Usdc micro={quote.fee} /></dd>
        </div>
        <div className="listing-row">
          <dt>Price valid until</dt>
          <dd>{deadline}</dd>
        </div>
      </dl>

      <div role="status" aria-live="polite" className="text-sm">
        {expired && !isFinalStatus(status ?? "") ? (
          <p className="muted">Quote expired. Request a new quote before sending funds.</p>
        ) : mirror ? (
          <div className="flex flex-col gap-2">
            <p>{mirrorSuccessMessage(mirror.usdc_micro)}</p>
            <p><TxLink sig={mirror.signature} /></p>
            <div className="flex flex-wrap items-center gap-2">
              <a href="#vault-side-head" className="btn-secondary">
                Contribute {formatUsdc(mirror.usdc_micro)} USDC
              </a>
              <span className="text-sm muted">Enter {formatUsdc(mirror.usdc_micro)} in the deposit form above.</span>
            </div>
          </div>
        ) : mirrorError ? (
          <div className="flex flex-col gap-1">
            <p>{swapStatusMessage(status ?? "")}</p>
            <p className="problem">{mirrorError}</p>
          </div>
        ) : status ? (
          <p>{swapStatusMessage(status)}</p>
        ) : null}
      </div>

      <p className="text-sm muted">
        Aurora runs on mainnet: this is real money. The USDC arrives at your Solana address on mainnet. This devnet demo vault cannot hold it; on mainnet it would go straight into the vault.
      </p>
    </div>
  );
}
