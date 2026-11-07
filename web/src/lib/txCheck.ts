// No blind signing: decode the API-built transaction and check it does exactly what the user asked before the wallet signs.
// Only Arrhabon and capped ComputeBudget top-level instructions, exactly one Arrhabon one, and no account outside it.
import {
  address,
  getAddressDecoder,
  getAddressEncoder,
  getCompiledTransactionMessageDecoder,
  getProgramDerivedAddress,
  getTransactionDecoder,
  getU64Encoder,
  type Address,
  type ReadonlyUint8Array,
} from "@solana/kit";
import { SYSTEM_PROGRAM_ADDRESS } from "@solana-program/system";
import { ASSOCIATED_TOKEN_PROGRAM_ADDRESS, TOKEN_PROGRAM_ADDRESS, findAssociatedTokenPda } from "@solana-program/token";
import { PROGRAM_ID } from "./config";
import { formatUsdc } from "./usdc";

const COMPUTE_BUDGET = "ComputeBudget111111111111111111111111111111";
/** Runtime ceiling for compute units per transaction. */
export const MAX_COMPUTE_UNITS = 1_400_000;
/** Priority-fee ceiling: 0.001 SOL. */
export const MAX_PRIORITY_FEE_LAMPORTS = 1_000_000n;

/** Anchor discriminators (sha256("global:<name>")[..8]) from target/idl/arrhabon.json. */
export const DISCRIMINATORS = {
  create_vault: [29, 237, 247, 208, 193, 82, 54, 135],
  contribute: [82, 33, 68, 131, 32, 0, 205, 95],
} as const;

/** What the user asked for; every argument and derived account in the transaction must match it. */
export type Intent =
  | {
      action: "create_vault";
      nonce: bigint;
      /** PDA ["vault", student, nonce LE], derived by the web, not taken from the API. */
      vault: string;
      university: string;
      attestor: string;
      target: bigint;
      /** Unix seconds. */
      deadline: bigint;
    }
  | { action: "contribute"; vault: string; amount: bigint };

/** Confirm-dialog text shown before the wallet signs. */
export function describeIntent(intent: Intent): string {
  if (intent.action === "contribute") return `Contribute ${formatUsdc(intent.amount)} USDC to vault ${intent.vault}?`;
  const deadline = new Date(Number(intent.deadline) * 1000).toLocaleString();
  return `Create vault ${intent.vault} with a target of ${formatUsdc(intent.target)} USDC and a visa deadline of ${deadline}?`;
}

const pda = async (seeds: (string | ReadonlyUint8Array)[], programId: string) =>
  (await getProgramDerivedAddress({ programAddress: address(programId), seeds }))[0];
const key = (a: string) => getAddressEncoder().encode(address(a));
const ata = async (owner: string, mint: string) =>
  (await findAssociatedTokenPda({ owner: address(owner), mint: address(mint), tokenProgram: TOKEN_PROGRAM_ADDRESS }))[0];

/** Same seeds as the program's create_vault: ["vault", student, nonce.to_le_bytes()]. */
export const vaultAddress = (student: string, nonce: bigint, programId: string = PROGRAM_ID) =>
  pda(["vault", key(student), getU64Encoder().encode(nonce)], programId);

const view = (data: ReadonlyUint8Array, offset: number, size: number) =>
  data.length < offset + size ? null : new DataView(data.buffer, data.byteOffset + offset, size);
const readU32 = (data: ReadonlyUint8Array, offset: number) => view(data, offset, 4)?.getUint32(0, true) ?? null;
const readU64 = (data: ReadonlyUint8Array, offset: number) => view(data, offset, 8)?.getBigUint64(0, true) ?? null;
const readI64 = (data: ReadonlyUint8Array, offset: number) => view(data, offset, 8)?.getBigInt64(0, true) ?? null;
const readKey = (data: ReadonlyUint8Array, offset: number) =>
  data.length < offset + 32 ? null : getAddressDecoder().decode(data.slice(offset, offset + 32));

type Expected = { accounts: Address[]; writable: Set<string>; dataLength: number };

/** Throws a user-facing Error when the transaction does not match the intent. */
export async function checkTx(txBytes: Uint8Array, wallet: string, intent: Intent, programId: string = PROGRAM_ID): Promise<void> {
  const refuse = (why: string): never => {
    throw new Error(`Refusing to sign: ${why}`);
  };
  let msg;
  try {
    msg = getCompiledTransactionMessageDecoder().decode(getTransactionDecoder().decode(txBytes).messageBytes);
  } catch {
    return refuse("the transaction from the API could not be decoded.");
  }
  // lookup tables would hide account addresses; the API never uses them, so refuse rather than resolve.
  if ("addressTableLookups" in msg && msg.addressTableLookups?.length) refuse("it uses address lookup tables.");
  const keys = msg.staticAccounts;
  if (keys[0] !== wallet) refuse(`the fee payer is ${keys[0]}, not your wallet.`);
  const { numSignerAccounts, numReadonlySignerAccounts, numReadonlyNonSignerAccounts } = msg.header;
  if (numSignerAccounts !== 1) refuse(`it asks for ${numSignerAccounts} signatures; only your wallet should sign.`);
  const isWritable = (i: number) =>
    i < numSignerAccounts ? i < numSignerAccounts - numReadonlySignerAccounts : i < keys.length - numReadonlyNonSignerAccounts;

  let unitLimit: number | null = null;
  let unitPrice: bigint | null = null;
  const ours = msg.instructions.filter((ix) => {
    const program = keys[ix.programAddressIndex];
    if (program === programId) return true;
    if (program !== COMPUTE_BUDGET) refuse(`it calls an unexpected program (${program}).`);
    // ComputeBudget: [kind u8, value]; only a capped unit limit (2, u32) and unit price (3, u64 micro-lamports).
    const data = ix.data ?? new Uint8Array();
    if (ix.accountIndices?.length) refuse("a compute budget instruction passes accounts.");
    if (data[0] === 2 && data.length === 5 && unitLimit === null) unitLimit = readU32(data, 1);
    else if (data[0] === 3 && data.length === 9 && unitPrice === null) unitPrice = readU64(data, 1);
    else refuse("it carries an unexpected or repeated compute budget instruction.");
    return false;
  });
  if (unitLimit !== null && unitLimit > MAX_COMPUTE_UNITS) refuse(`it asks for ${unitLimit} compute units (max ${MAX_COMPUTE_UNITS}).`);
  // micro-lamports per unit x units / 1e6; without a limit assume the runtime maximum.
  const fee = ((unitPrice ?? 0n) * BigInt(unitLimit ?? MAX_COMPUTE_UNITS)) / 1_000_000n;
  if (fee > MAX_PRIORITY_FEE_LAMPORTS) refuse(`its priority fee is ${fee} lamports (max ${MAX_PRIORITY_FEE_LAMPORTS}).`);
  if (ours.length !== 1) refuse(`expected one Arrhabon instruction, found ${ours.length}.`);

  const ix = ours[0];
  const data = ix.data ?? new Uint8Array();
  const accounts = (ix.accountIndices ?? []).map((i) => keys[i]);
  const disc = DISCRIMINATORS[intent.action];
  if (data.length < 8 || disc.some((b, i) => data[i] !== b)) refuse(`it is not a ${intent.action} instruction.`);
  // the mint is pinned on-chain (config.mint / vault.mint); every account derived from it is checked here.
  const expected = intent.action === "contribute"
    ? await checkContribute(data, accounts, wallet, intent, programId, refuse)
    : await checkCreateVault(data, accounts, wallet, intent, programId, refuse);

  if (data.length !== expected.dataLength) refuse("the instruction carries unexpected extra data.");
  if (accounts.length !== expected.accounts.length) refuse(`it passes ${accounts.length} accounts, expected ${expected.accounts.length}.`);
  expected.accounts.forEach((a, i) => accounts[i] !== a && refuse(`account ${i} is ${accounts[i]}, expected ${a}.`));
  const allowed = new Set([programId, COMPUTE_BUDGET, ...expected.accounts]);
  keys.forEach((k, i) => {
    if (!allowed.has(k)) refuse(`it touches an account outside your request (${k}).`);
    if (isWritable(i) && !expected.writable.has(k)) refuse(`it would modify an account outside your request (${k}).`);
  });
}

type Refuse = (why: string) => never;

async function checkContribute(
  data: ReadonlyUint8Array,
  accounts: string[],
  wallet: string,
  intent: Extract<Intent, { action: "contribute" }>,
  programId: string,
  refuse: Refuse,
): Promise<Expected> {
  // accounts: funder, vault, contribution, mint, funder_token, vault_token, token_program, system_program; args: amount u64.
  if (accounts[1] !== intent.vault) refuse(`it contributes to vault ${accounts[1]}, not ${intent.vault}.`);
  const amount = readU64(data, 8);
  if (amount !== intent.amount) refuse(`the amount is ${amount} micro-USDC, not ${intent.amount}.`);
  const mint = accounts[3] ?? refuse("the mint account is missing.");
  const [contribution, funderToken, vaultToken] = await Promise.all([
    pda(["contrib", key(intent.vault), key(wallet)], programId),
    ata(wallet, mint),
    ata(intent.vault, mint),
  ]);
  return {
    accounts: [address(wallet), address(intent.vault), contribution, address(mint), funderToken, vaultToken, TOKEN_PROGRAM_ADDRESS, SYSTEM_PROGRAM_ADDRESS],
    writable: new Set([wallet, intent.vault, contribution, funderToken, vaultToken]),
    dataLength: 8 + 8,
  };
}

async function checkCreateVault(
  data: ReadonlyUint8Array,
  accounts: string[],
  wallet: string,
  intent: Extract<Intent, { action: "create_vault" }>,
  programId: string,
  refuse: Refuse,
): Promise<Expected> {
  // args after the discriminator: nonce u64 @8, university @16, attestor @48 (32 bytes each), target u64 @80, deadline i64 @88.
  const nonce = readU64(data, 8);
  if (nonce !== intent.nonce) refuse(`the vault nonce is ${nonce}, not ${intent.nonce}.`);
  const university = readKey(data, 16);
  if (university !== intent.university) refuse(`the university is ${university}, not ${intent.university}.`);
  const attestor = readKey(data, 48);
  if (attestor !== intent.attestor) refuse(`the attestor is ${attestor}, not ${intent.attestor}.`);
  const target = readU64(data, 80);
  if (target !== intent.target) refuse(`the target is ${target} micro-USDC, not ${intent.target}.`);
  const deadline = readI64(data, 88);
  if (deadline !== intent.deadline) refuse(`the deadline is ${deadline}, not ${intent.deadline} (unix seconds).`);
  const vault = await vaultAddress(wallet, intent.nonce, programId);
  if (vault !== intent.vault) refuse(`your vault address should be ${vault}, not ${intent.vault}.`);
  if (accounts[1] !== vault) refuse(`it creates vault ${accounts[1]}, not ${vault}.`);
  // accounts: student, vault, config, registered_university, registered_attestor, mint, vault_token, token, ata, system.
  const mint = accounts[5] ?? refuse("the mint account is missing.");
  const [config, registeredUniversity, registeredAttestor, vaultToken] = await Promise.all([
    pda(["config"], programId),
    pda(["university", key(intent.university)], programId),
    pda(["attestor", key(intent.attestor)], programId),
    ata(vault, mint),
  ]);
  return {
    accounts: [
      address(wallet), vault, config, registeredUniversity, registeredAttestor, address(mint), vaultToken,
      TOKEN_PROGRAM_ADDRESS, ASSOCIATED_TOKEN_PROGRAM_ADDRESS, SYSTEM_PROGRAM_ADDRESS,
    ],
    writable: new Set([wallet, vault, vaultToken]),
    dataLength: 8 + 8 + 32 + 32 + 8 + 8,
  };
}
