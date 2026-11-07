import { describe, expect, it } from "vitest";
import {
  AccountRole,
  address,
  appendTransactionMessageInstructions,
  blockhash,
  compileTransaction,
  createTransactionMessage,
  decompileTransactionMessage,
  getCompiledTransactionMessageDecoder,
  getTransactionDecoder,
  getTransactionEncoder,
  pipe,
  setTransactionMessageFeePayer,
  setTransactionMessageLifetimeUsingBlockhash,
  type Instruction,
} from "@solana/kit";
import { DISCRIMINATORS, checkTx, describeIntent, vaultAddress } from "./txCheck";

const WALLET = "9xQeWvG816bUx9EPjHmaT23yvVM2ZWbrrpZb9PusVFin";
const OTHER = "4Nd1mBQtrMJVYVfKf2PJy9NZUZdTAsp7D4xWLs4gDB4T";
const UNIVERSITY = "7EYnhQoR9YM3N7UoaKRoA44Uy8JeaZV3qyouov87awMs";
const SYSTEM = "11111111111111111111111111111111";
const COMPUTE_BUDGET = "ComputeBudget111111111111111111111111111111";
const VAULT = "FRocS2FwP3Xk4Dd9L52Y5ksWErMbLeiCPk6vzfXk4TfS";

// Unsigned legacy transactions from the API's Rust builder (crates/solana_adapter instructions::unsigned_tx) for
// student/funder WALLET, nonce 1762000000000, university UNIVERSITY, attestor OTHER, target 1,000 USDC,
// deadline 1767225600, contribution 250 USDC to a vault naming UNIVERSITY and attestor OTHER, devnet USDC mint, zero blockhash.
const hex = (h: string) => Uint8Array.from(h.match(/../g)!, (b) => parseInt(b, 16));
const API_CREATE = hex(
  "01" + "00".repeat(64) + "0100080b850f2d6e02a47af824d09ab69dc42d70cb28cbfa249fb7ee57b9d256c12762efd65e426d8a49739e1118149cc50a21b906cca4ba0210cbe4d13751cabb200e45fe130719b5aacf935e65e229b6b8f8eeaac834f0d8d27adbaebbbecd5c4e13de000000000000000000000000000000000000000000000000000000000000000006ddf6e1d765a193d9cbe146ceeb79ac1cb485ed5f5b37913a8cf5857eff00a916d84c99bde249074f240517cae820ef92b6f332364a774fcffdd0387bfb962e1923bd3d1374a869bce19ddaf02ffd673caf6d16ea11e9f3ccaebe73a7c0af47286e3b5658abe65d11d13c97ef1239df152fbc6842312fdfa8a5c49f6310fee43b442cb3912157f13a933d0134282d032b5ffecd01a2dbf1b7790608df002ea78c97258f4e2489f1bb3d1029148e0d830b5a1399daff1084048e7bd8dbe9f859944b3aed7bf2641b315bba0bf2b65861781c50da6be5306b00fed8bad634e131000000000000000000000000000000000000000000000000000000000000000001070a000105060a0802040903601dedf7d0c15236870054623f9a0100005c9e9d5fa309864acd7d8118d326313cde7b86df18e197924a03186ffe9f3d7a321cfa5add185e8893a5fd88013ec4d7e122ded46354cadff50d956395e75b6000ca9a3b0000000000b9556900000000",
);
const API_CONTRIBUTE = hex(
  "01" + "00".repeat(64) + "0100060b850f2d6e02a47af824d09ab69dc42d70cb28cbfa249fb7ee57b9d256c12762ef400e79133ce242e1b4343c40f0c372e25d75747a61ec08abd9118815250bdabdcba9a6c3633e124547a8bb0d61c0607f38c465bbb222157b094619bde99fad6dd65e426d8a49739e1118149cc50a21b906cca4ba0210cbe4d13751cabb200e45fe130719b5aacf935e65e229b6b8f8eeaac834f0d8d27adbaebbbecd5c4e13de000000000000000000000000000000000000000000000000000000000000000006ddf6e1d765a193d9cbe146ceeb79ac1cb485ed5f5b37913a8cf5857eff00a91923bd3d1374a869bce19ddaf02ffd673caf6d16ea11e9f3ccaebe73a7c0af47286e3b5658abe65d11d13c97ef1239df152fbc6842312fdfa8a5c49f6310fee43b442cb3912157f13a933d0134282d032b5ffecd01a2dbf1b7790608df002ea7944b3aed7bf2641b315bba0bf2b65861781c50da6be5306b00fed8bad634e131000000000000000000000000000000000000000000000000000000000000000001080a0003070a02090104060510522144832000cd5f80b2e60e00000000",
);

const create = {
  action: "create_vault",
  nonce: 1762000000000n,
  vault: VAULT,
  university: UNIVERSITY,
  attestor: OTHER,
  target: 1_000_000_000n,
  deadline: 1767225600n,
} as const;
const contribute = { action: "contribute", vault: VAULT, university: UNIVERSITY, attestor: OTHER, amount: 250_000_000n } as const;

/** The instructions inside an API transaction, to tamper with and recompile. */
function ixsOf(bytes: Uint8Array) {
  const compiled = getCompiledTransactionMessageDecoder().decode(getTransactionDecoder().decode(bytes).messageBytes);
  return [...decompileTransactionMessage(compiled as Parameters<typeof decompileTransactionMessage>[0]).instructions];
}
const arrhabon = (bytes: Uint8Array) => ixsOf(bytes)[0];

function tx(feePayer: string, ixs: Instruction[]) {
  const msg = pipe(
    createTransactionMessage({ version: 0 }),
    (m) => setTransactionMessageFeePayer(address(feePayer), m),
    (m) => setTransactionMessageLifetimeUsingBlockhash({ blockhash: blockhash(SYSTEM), lastValidBlockHeight: 100n }, m),
    (m) => appendTransactionMessageInstructions(ixs, m),
  );
  return new Uint8Array(getTransactionEncoder().encode(compileTransaction(msg)));
}

const withData = (ix: Instruction, patch: (d: Uint8Array) => void): Instruction => {
  const data = new Uint8Array(ix.data!);
  patch(data);
  return { ...ix, data };
};
const setU64 = (d: Uint8Array, offset: number, n: bigint) => new DataView(d.buffer).setBigUint64(offset, n, true);
const withAccount = (ix: Instruction, i: number, a: string, role = ix.accounts![i].role): Instruction => ({
  ...ix,
  accounts: ix.accounts!.map((m, j) => (j === i ? { address: address(a), role } : m)),
});

const budget = (kind: number, value: bigint, size: number): Instruction => {
  const data = new Uint8Array(1 + size);
  data[0] = kind;
  const v = new DataView(data.buffer);
  if (size === 4) v.setUint32(1, Number(value), true);
  else v.setBigUint64(1, value, true);
  return { programAddress: address(COMPUTE_BUDGET), data };
};
const unitLimit = (n: number) => budget(2, BigInt(n), 4);
const unitPrice = (microLamports: bigint) => budget(3, microLamports, 8);

describe("checkTx: honest API transactions", () => {
  it("derives the vault with the program's seeds", async () => {
    expect(await vaultAddress(WALLET, create.nonce)).toBe(VAULT);
  });
  it("accepts the API's create_vault and contribute", async () => {
    await expect(checkTx(API_CREATE, WALLET, create)).resolves.toBeUndefined();
    await expect(checkTx(API_CONTRIBUTE, WALLET, contribute)).resolves.toBeUndefined();
  });
  it("accepts a capped unit limit and unit price", async () => {
    const bytes = tx(WALLET, [unitLimit(200_000), unitPrice(5_000n), arrhabon(API_CONTRIBUTE)]);
    await expect(checkTx(bytes, WALLET, contribute)).resolves.toBeUndefined();
  });
});

describe("checkTx: create_vault", () => {
  it("refuses a target, university or attestor other than requested", async () => {
    await expect(checkTx(API_CREATE, WALLET, { ...create, target: 1n })).rejects.toThrow(/target/);
    await expect(checkTx(API_CREATE, WALLET, { ...create, university: OTHER })).rejects.toThrow(/university/);
    await expect(checkTx(API_CREATE, WALLET, { ...create, attestor: UNIVERSITY })).rejects.toThrow(/attestor/);
  });
  it("refuses a tampered deadline", async () => {
    const bytes = tx(WALLET, [withData(arrhabon(API_CREATE), (d) => setU64(d, 88, 1767225600n + 86_400n * 365n))]);
    await expect(checkTx(bytes, WALLET, create)).rejects.toThrow(/deadline is 1798761600, not 1767225600/);
  });
  it("refuses a wrong nonce or vault address", async () => {
    await expect(checkTx(API_CREATE, WALLET, { ...create, nonce: 1n })).rejects.toThrow(/nonce/);
    await expect(checkTx(API_CREATE, WALLET, { ...create, vault: OTHER })).rejects.toThrow(/vault address/);
    const otherVault = await vaultAddress(WALLET, 1n);
    const bytes = tx(WALLET, [withAccount(arrhabon(API_CREATE), 1, otherVault)]);
    await expect(checkTx(bytes, WALLET, create)).rejects.toThrow(/creates vault/);
  });
  it("refuses a swapped derived account", async () => {
    const bytes = tx(WALLET, [withAccount(arrhabon(API_CREATE), 6, OTHER)]);
    await expect(checkTx(bytes, WALLET, create)).rejects.toThrow(/account 6/);
  });
  it("refuses a truncated or padded instruction", async () => {
    const truncated = tx(WALLET, [{ ...arrhabon(API_CREATE), data: arrhabon(API_CREATE).data!.slice(0, 16) }]);
    await expect(checkTx(truncated, WALLET, create)).rejects.toThrow(/university/);
    const padded = tx(WALLET, [{ ...arrhabon(API_CREATE), data: new Uint8Array([...arrhabon(API_CREATE).data!, 0]) }]);
    await expect(checkTx(padded, WALLET, create)).rejects.toThrow(/extra data/);
  });
  it("describes the vault, target and deadline", () => {
    const text = describeIntent(create);
    expect(text).toContain(VAULT);
    expect(text).toContain("1,000.00 USDC");
    expect(text).toContain(new Date(1767225600 * 1000).toLocaleString());
  });
});

describe("checkTx: contribute", () => {
  it("refuses the wrong vault or amount", async () => {
    await expect(checkTx(API_CONTRIBUTE, WALLET, { ...contribute, vault: OTHER })).rejects.toThrow(/vault/);
    await expect(checkTx(API_CONTRIBUTE, WALLET, { ...contribute, amount: 1n })).rejects.toThrow(/amount/);
  });
  it("refuses a funder token account that is not the funder's", async () => {
    const bytes = tx(WALLET, [withAccount(arrhabon(API_CONTRIBUTE), 6, OTHER)]);
    await expect(checkTx(bytes, WALLET, contribute)).rejects.toThrow(/account 6/);
  });
  it("refuses a registry entry other than the vault university's", async () => {
    const bytes = tx(WALLET, [withAccount(arrhabon(API_CONTRIBUTE), 2, OTHER)]);
    await expect(checkTx(bytes, WALLET, contribute)).rejects.toThrow(/account 2/);
    await expect(checkTx(API_CONTRIBUTE, WALLET, { ...contribute, university: OTHER })).rejects.toThrow(/account 2/);
  });
  it("refuses an attestor registry entry other than the vault attestor's", async () => {
    const bytes = tx(WALLET, [withAccount(arrhabon(API_CONTRIBUTE), 3, UNIVERSITY)]);
    await expect(checkTx(bytes, WALLET, contribute)).rejects.toThrow(/account 3/);
    await expect(checkTx(API_CONTRIBUTE, WALLET, { ...contribute, attestor: WALLET })).rejects.toThrow(/account 3/);
  });
  it("refuses a writable registry entry", async () => {
    const ix = arrhabon(API_CONTRIBUTE);
    for (const i of [2, 3]) {
      const bytes = tx(WALLET, [withAccount(ix, i, ix.accounts![i].address, AccountRole.WRITABLE)]);
      await expect(checkTx(bytes, WALLET, contribute)).rejects.toThrow(/modify an account/);
    }
  });
  it("refuses extra accounts on the instruction", async () => {
    const ix = arrhabon(API_CONTRIBUTE);
    const bytes = tx(WALLET, [{ ...ix, accounts: [...ix.accounts!, { address: address(OTHER), role: AccountRole.WRITABLE }] }]);
    await expect(checkTx(bytes, WALLET, contribute)).rejects.toThrow(/passes 11 accounts/);
  });
  it("describes the intent in USDC", () => {
    expect(describeIntent(contribute)).toBe(`Contribute 250.00 USDC to vault ${VAULT}?`);
  });
});

describe("checkTx: compute budget and envelope", () => {
  it("refuses an oversized unit limit or priority fee", async () => {
    const big = tx(WALLET, [unitLimit(1_400_001), arrhabon(API_CONTRIBUTE)]);
    await expect(checkTx(big, WALLET, contribute)).rejects.toThrow(/compute units/);
    // 1e9 micro-lamports x 200k units = 0.2 SOL
    const pricey = tx(WALLET, [unitLimit(200_000), unitPrice(1_000_000_000n), arrhabon(API_CONTRIBUTE)]);
    await expect(checkTx(pricey, WALLET, contribute)).rejects.toThrow(/priority fee/);
    // no limit: assume the 1.4M maximum, so 1e6 micro-lamports = 1.4M lamports
    const noLimit = tx(WALLET, [unitPrice(1_000_000n), arrhabon(API_CONTRIBUTE)]);
    await expect(checkTx(noLimit, WALLET, contribute)).rejects.toThrow(/priority fee/);
  });
  it("refuses a repeated, heap-frame or malformed compute budget instruction", async () => {
    const twice = tx(WALLET, [unitPrice(1n), unitPrice(1n), arrhabon(API_CONTRIBUTE)]);
    await expect(checkTx(twice, WALLET, contribute)).rejects.toThrow(/compute budget/);
    const heap = tx(WALLET, [budget(1, 256n * 1024n, 4), arrhabon(API_CONTRIBUTE)]);
    await expect(checkTx(heap, WALLET, contribute)).rejects.toThrow(/compute budget/);
    const bare = tx(WALLET, [{ programAddress: address(COMPUTE_BUDGET), data: new Uint8Array([2]) }, arrhabon(API_CONTRIBUTE)]);
    await expect(checkTx(bare, WALLET, contribute)).rejects.toThrow(/compute budget/);
  });
  it("refuses a different fee payer or an extra signer", async () => {
    await expect(checkTx(tx(OTHER, [arrhabon(API_CONTRIBUTE)]), WALLET, contribute)).rejects.toThrow(/fee payer/);
    const ix = withAccount(arrhabon(API_CONTRIBUTE), 6, OTHER, AccountRole.READONLY_SIGNER);
    await expect(checkTx(tx(WALLET, [ix]), WALLET, contribute)).rejects.toThrow(/signatures/);
  });
  it("refuses a foreign program or a second Arrhabon instruction", async () => {
    const foreign = tx(WALLET, [{ programAddress: address(SYSTEM), data: new Uint8Array([2, 0, 0, 0]) }, arrhabon(API_CONTRIBUTE)]);
    await expect(checkTx(foreign, WALLET, contribute)).rejects.toThrow(/unexpected program/);
    const two = tx(WALLET, [arrhabon(API_CONTRIBUTE), arrhabon(API_CONTRIBUTE)]);
    await expect(checkTx(two, WALLET, contribute)).rejects.toThrow(/found 2/);
    await expect(checkTx(tx(WALLET, [unitLimit(1)]), WALLET, contribute)).rejects.toThrow(/found 0/);
  });
  it("refuses a program account marked writable", async () => {
    const bytes = tx(WALLET, [withAccount(arrhabon(API_CONTRIBUTE), 9, SYSTEM, AccountRole.WRITABLE)]);
    await expect(checkTx(bytes, WALLET, contribute)).rejects.toThrow(/modify an account/);
  });
  it("refuses the wrong action and garbage bytes", async () => {
    await expect(checkTx(API_CONTRIBUTE, WALLET, create)).rejects.toThrow(/create_vault/);
    expect(DISCRIMINATORS.create_vault).toHaveLength(8);
    await expect(checkTx(new Uint8Array([1, 2, 3]), WALLET, contribute)).rejects.toThrow(/decoded/);
  });
});
