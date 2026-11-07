"use client";
import { useEffect, useId, useState, type FormEvent } from "react";
import Link from "next/link";
import { api } from "@/lib/api";
import { ATTESTORS, UNIVERSITIES, nameParties, type Party } from "@/lib/config";
import { parseUsdc } from "@/lib/usdc";
import { vaultAddress } from "@/lib/txCheck";
import { useWallet } from "./wallet";
import { useSignAndSubmit } from "./useSignAndSubmit";
import { Card, Problem } from "./parts";
import { Field, TxStatus } from "./ui";

function PartySelect({ label, name, parties }: { label: string; name: string; parties: Party[] }) {
  return (
    <Field label={label}>
      {parties.length ? (
        <select name={name} required className="input w-full max-w-full">
          {parties.map((p) => (
            <option key={p.address} value={p.address}>
              {p.name}
            </option>
          ))}
        </select>
      ) : (
        <input name={name} required className="input w-full max-w-full font-mono" placeholder="Registered Solana address" />
      )}
    </Field>
  );
}

export function CreateVaultForm({ onCreated }: { onCreated: () => void }) {
  const wallet = useWallet();
  const tx = useSignAndSubmit();
  const [vault, setVault] = useState<string | null>(null);
  const [inputError, setInputError] = useState<string | null>(null);
  const [parties, setParties] = useState({ universities: UNIVERSITIES, attestors: ATTESTORS });
  const why = useId();

  useEffect(() => {
    api
      .registry()
      .then((r) =>
        setParties({
          universities: nameParties(r.universities.map((u) => u.address), UNIVERSITIES),
          attestors: nameParties(r.attestors.map((a) => a.address), ATTESTORS),
        }),
      )
      .catch(() => {}); // registry unavailable: keep the env-configured parties
  }, []);

  async function submit(e: FormEvent<HTMLFormElement>) {
    e.preventDefault();
    const f = new FormData(e.currentTarget);
    const target = parseUsdc(String(f.get("target")));
    const deadline = Math.floor(new Date(String(f.get("deadline"))).getTime() / 1000);
    if (target === null || !Number.isFinite(deadline)) return setInputError("Enter a USDC amount and a deadline.");
    if (!wallet.address) return setInputError("Log in to create a vault.");
    setInputError(null);
    const nonce = Date.now();
    const expected = await vaultAddress(wallet.address, BigInt(nonce));
    const university = String(f.get("university"));
    const attestor = String(f.get("attestor"));
    const done = await tx.run(
      () =>
        api.createVault({
          student: wallet.address!,
          university,
          attestor,
          target: Number(target),
          deadline,
          nonce,
        }),
      { action: "create_vault", nonce: BigInt(nonce), vault: expected, university, attestor, target, deadline: BigInt(deadline) },
    );
    if (done) {
      setVault(expected);
      onCreated();
    }
  }

  const share = vault ? `${location.origin}/vault/${vault}` : "";
  const disabledWhy = !wallet.address ? "Log in to create a vault." : tx.busy ? "Approve the transaction in your wallet to continue." : null;
  return (
    <Card title="Create a vault">
      <form onSubmit={submit} className="grid grid-cols-1 gap-4 sm:grid-cols-2">
        <PartySelect label="University" name="university" parties={parties.universities} />
        <PartySelect label="Visa attestor (confirms the visa decision)" name="attestor" parties={parties.attestors} />
        <div className="flex min-w-0 flex-col gap-1">
          <Field label="Target (USDC)">
            <input name="target" required inputMode="decimal" className="input w-full max-w-full" placeholder="1000" aria-describedby={`${why}-target`} />
          </Field>
          <p id={`${why}-target`} className="muted text-sm">The university is paid only if at least this much is held.</p>
        </div>
        <div className="flex min-w-0 flex-col gap-1">
          <Field label="Deadline for the visa decision (your local time)">
            <input name="deadline" required type="datetime-local" className="input w-full max-w-full" aria-describedby={`${why}-deadline`} />
          </Field>
          <p id={`${why}-deadline`} className="muted text-sm">No visa confirmed by then: every funder is refunded.</p>
        </div>
        <div className="flex min-w-0 flex-col gap-2 sm:col-span-2">
          <button className="btn self-start" disabled={disabledWhy !== null} aria-describedby={disabledWhy ? why : undefined}>
            {tx.busy ? "Waiting for signature…" : "Create vault"}
          </button>
          {disabledWhy && <p id={why} className="text-sm">{disabledWhy}</p>}
          <Problem error={inputError ?? tx.error} />
          <TxStatus notice={tx.notice} sig={tx.signature} />
          {vault && (
            <p className="text-sm">
              Share with your funders: <Link className="link break-all" href={`/vault/${vault}`}>{share}</Link>
            </p>
          )}
        </div>
      </form>
    </Card>
  );
}
