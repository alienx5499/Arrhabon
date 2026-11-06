import { describe, expect, it } from "vitest";
import {
  address,
  appendTransactionMessageInstructions,
  blockhash,
  compileTransaction,
  createTransactionMessage,
  getAddressEncoder,
  getTransactionEncoder,
  pipe,
  setTransactionMessageFeePayer,
  setTransactionMessageLifetimeUsingBlockhash,
  type Instruction,
} from "@solana/kit";
import { DISCRIMINATORS, checkTx, describeIntent } from "./txCheck";
import { PROGRAM_ID } from "./config";

const WALLET = "9xQeWvG816bUx9EPjHmaT23yvVM2ZWbrrpZb9PusVFin";
const OTHER = "4Nd1mBQtrMJVYVfKf2PJy9NZUZdTAsp7D4xWLs4gDB4T";
const VAULT = "7EYnhQoR9YM3N7UoaKRoA44Uy8JeaZV3qyouov87awMs";
const SYSTEM = "11111111111111111111111111111111";

const u64 = (n: bigint) => {
  const b = new Uint8Array(8);
  new DataView(b.buffer).setBigUint64(0, n, true);
  return b;
};

function ix(program: string, disc: readonly number[], accounts: string[] = [], args = new Uint8Array()): Instruction {
  return {
    programAddress: address(program),
    accounts: accounts.map((a) => ({ address: address(a), role: 0 })),
    data: new Uint8Array([...disc, ...args]),
  };
}

const contributeIx = (vault = VAULT, amount = 250_000_000n) =>
  ix(PROGRAM_ID, DISCRIMINATORS.contribute, [WALLET, vault], u64(amount));

function tx(feePayer: string, ixs: Instruction[]) {
  const msg = pipe(
    createTransactionMessage({ version: 0 }),
    (m) => setTransactionMessageFeePayer(address(feePayer), m),
    (m) => setTransactionMessageLifetimeUsingBlockhash({ blockhash: blockhash(SYSTEM), lastValidBlockHeight: 100n }, m),
    (m) => appendTransactionMessageInstructions(ixs, m),
  );
  return new Uint8Array(getTransactionEncoder().encode(compileTransaction(msg)));
}

const contribute = { action: "contribute", vault: VAULT, amount: 250_000_000n } as const;

describe("checkTx", () => {
  it("accepts a matching contribute with a ComputeBudget instruction", () => {
    const bytes = tx(WALLET, [ix("ComputeBudget111111111111111111111111111111", [2]), contributeIx()]);
    expect(() => checkTx(bytes, WALLET, contribute)).not.toThrow();
  });
  it("checks create_vault target, university and attestor", () => {
    const key = (a: string) => getAddressEncoder().encode(address(a));
    const args = new Uint8Array([...u64(7n), ...key(VAULT), ...key(OTHER), ...u64(1_000_000_000n), ...u64(0n)]);
    const bytes = tx(WALLET, [ix(PROGRAM_ID, DISCRIMINATORS.create_vault, [WALLET], args)]);
    const create = { action: "create_vault", target: 1_000_000_000n, university: VAULT, attestor: OTHER } as const;
    expect(() => checkTx(bytes, WALLET, create)).not.toThrow();
    expect(() => checkTx(bytes, WALLET, { ...create, target: 1n })).toThrow(/target/);
    expect(() => checkTx(bytes, WALLET, { ...create, university: OTHER })).toThrow(/university/);
    expect(() => checkTx(bytes, WALLET, { ...create, attestor: VAULT })).toThrow(/attestor/);
  });
  it("refuses a truncated create_vault", () => {
    const bytes = tx(WALLET, [ix(PROGRAM_ID, DISCRIMINATORS.create_vault, [WALLET], u64(7n))]);
    const create = { action: "create_vault", target: 1n, university: VAULT, attestor: OTHER } as const;
    expect(() => checkTx(bytes, WALLET, create)).toThrow(/university/);
  });
  it("refuses a different fee payer", () => {
    expect(() => checkTx(tx(OTHER, [contributeIx()]), WALLET, contribute)).toThrow(/fee payer/);
  });
  it("refuses a foreign program", () => {
    const bytes = tx(WALLET, [ix(SYSTEM, [2, 0, 0, 0]), contributeIx()]);
    expect(() => checkTx(bytes, WALLET, contribute)).toThrow(/unexpected program/);
  });
  it("refuses a transaction with no Arrhabon instruction", () => {
    const bytes = tx(WALLET, [ix("ComputeBudget111111111111111111111111111111", [2])]);
    expect(() => checkTx(bytes, WALLET, contribute)).toThrow(/found 0/);
  });
  it("refuses the wrong action", () => {
    expect(() => checkTx(tx(WALLET, [contributeIx()]), WALLET, { action: "create_vault", target: 1n, university: VAULT, attestor: OTHER })).toThrow(/create_vault/);
  });
  it("refuses the wrong vault or amount", () => {
    expect(() => checkTx(tx(WALLET, [contributeIx(OTHER)]), WALLET, contribute)).toThrow(/vault/);
    expect(() => checkTx(tx(WALLET, [contributeIx(VAULT, 1n)]), WALLET, contribute)).toThrow(/amount/);
  });
  it("describes the intent in USDC", () => {
    expect(describeIntent(contribute)).toBe(`Contribute 250.00 USDC to vault ${VAULT}?`);
  });
  it("refuses garbage bytes", () => {
    expect(() => checkTx(new Uint8Array([1, 2, 3]), WALLET, contribute)).toThrow(/decoded/);
  });
});
