"use client";
// Login controls for wallet routes. The header lives in the root layout, outside the wallet provider, so these render into its slot.
import { useSyncExternalStore } from "react";
import { createPortal } from "react-dom";
import { useWallet } from "./wallet";
import { AddressLink } from "./ui";
import { ACCOUNT_SLOT_ID } from "./LoginBar";
import { TestFunds } from "./TestFunds";

const noSubscribe = () => () => {};

export function Account() {
  const w = useWallet();
  // null on the server and during hydration, the header slot afterwards.
  const slot = useSyncExternalStore(noSubscribe, () => document.getElementById(ACCOUNT_SLOT_ID), () => null);
  if (!slot || !w.enabled) return null;
  return createPortal(
    w.address ? (
      <>
        <AddressLink address={w.address} />
        <TestFunds address={w.address} />
        <button className="btn-secondary" onClick={w.logout}>Log out</button>
      </>
    ) : (
      <button className="btn-secondary" onClick={w.login} disabled={!w.ready}>Log in</button>
    ),
    slot,
  );
}
