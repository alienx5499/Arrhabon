"use client";
import { useEffect, useState, type FormEvent } from "react";
import Link from "next/link";
import { api } from "@/lib/api";
import { ATTESTORS, UNIVERSITIES, nameParties, type Party } from "@/lib/config";
import { parseUsdc } from "@/lib/usdc";
import { useWallet } from "./wallet";
import { useSignAndSubmit } from "./useSignAndSubmit";
import { Card, ErrorText, Field, TxStatus } from "./ui";

function PartySelect({ label, name, parties }: { label: string; name: string; parties: Party[] }) {
  return (
    <Field label={label}>
      {parties.length ? (
        <select name={name} required className="input">
          {parties.map((p) => (
            <option key={p.address} value={p.address}>
              {p.name}
            </option>
          ))}
        </select>
      ) : (
        <input name={name} required className="input font-mono" placeholder="Registered base58 address" />
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
    setInputError(null);
    const nonce = Date.now();
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
      { action: "create_vault", target, university, attestor },
      (built) => built.vault,
    );
    if (done) {
      setVault(done.built.vault);
      onCreated();
    }
  }

  const share = vault ? `${location.origin}/vault/${vault}` : "";
  return (
    <Card title="Create a vault">
      <form onSubmit={submit} className="grid gap-4 sm:grid-cols-2">
        <PartySelect label="University" name="university" parties={parties.universities} />
        <PartySelect label="Visa attestor" name="attestor" parties={parties.attestors} />
        <Field label="Target (USDC)">
          <input name="target" required inputMode="decimal" className="input" placeholder="1000" />
        </Field>
        <Field label="Deadline for the visa decision">
          <input name="deadline" required type="datetime-local" className="input" />
        </Field>
        <div className="flex flex-col gap-2 sm:col-span-2">
          <button className="btn self-start" disabled={!wallet.address || tx.busy}>
            {tx.busy ? "Waiting for signature…" : "Create vault"}
          </button>
          {!wallet.address && <p className="text-sm">Log in to create a vault.</p>}
          <ErrorText error={inputError ?? tx.error} />
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
