// No blind signing: decode the API-built transaction and check it does what the user asked before the wallet signs.
// Mirrors the API relay rule: only Arrhabon and ComputeBudget top-level instructions, at least one Arrhabon one.
import {
  getAddressDecoder,
  getCompiledTransactionMessageDecoder,
  getTransactionDecoder,
  type ReadonlyUint8Array,
} from "@solana/kit";
import { PROGRAM_ID } from "./config";
import { formatUsdc } from "./usdc";

const COMPUTE_BUDGET = "ComputeBudget111111111111111111111111111111";

/** Anchor discriminators (sha256("global:<name>")[..8]) from target/idl/arrhabon.json. */
export const DISCRIMINATORS = {
  create_vault: [29, 237, 247, 208, 193, 82, 54, 135],
  contribute: [82, 33, 68, 131, 32, 0, 205, 95],
} as const;

export type Intent =
  | { action: "create_vault"; target: bigint; university: string; attestor: string }
  | { action: "contribute"; vault: string; amount: bigint };

/** Confirm-dialog text shown before the wallet signs. */
export function describeIntent(intent: Intent, vault?: string): string {
  return intent.action === "contribute"
    ? `Contribute ${formatUsdc(intent.amount)} USDC to vault ${intent.vault}?`
    : `Create vault ${vault ?? ""} with a target of ${formatUsdc(intent.target)} USDC?`;
}

const readU64 = (data: ReadonlyUint8Array, offset: number) =>
  data.length < offset + 8 ? null : new DataView(data.buffer, data.byteOffset + offset, 8).getBigUint64(0, true);
const readKey = (data: ReadonlyUint8Array, offset: number) =>
  data.length < offset + 32 ? null : getAddressDecoder().decode(data.slice(offset, offset + 32));

/** Throws a user-facing Error when the transaction does not match the intent. */
export function checkTx(txBytes: Uint8Array, wallet: string, intent: Intent, programId: string = PROGRAM_ID): void {
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

  const ours = msg.instructions.filter((ix) => {
    const program = keys[ix.programAddressIndex];
    if (program === programId) return true;
    if (program !== COMPUTE_BUDGET) refuse(`it calls an unexpected program (${program}).`);
    return false;
  });
  if (ours.length !== 1) refuse(`expected one Arrhabon instruction, found ${ours.length}.`);

  const ix = ours[0];
  const data = ix.data ?? new Uint8Array();
  const disc = DISCRIMINATORS[intent.action];
  if (data.length < 8 || disc.some((b, i) => data[i] !== b)) refuse(`it is not a ${intent.action} instruction.`);
  if (intent.action === "contribute") {
    // contribute accounts: [funder, vault, ...]; args: amount u64 LE after the discriminator.
    const vault = keys[ix.accountIndices?.[1] ?? -1];
    if (vault !== intent.vault) refuse(`it contributes to vault ${vault}, not ${intent.vault}.`);
    const amount = readU64(data, 8);
    if (amount !== intent.amount) refuse(`the amount is ${amount} micro-USDC, not ${intent.amount}.`);
  } else {
    // create_vault args after the discriminator: nonce u64, university, attestor (32 bytes each), target u64.
    const university = readKey(data, 16);
    if (university !== intent.university) refuse(`the university is ${university}, not ${intent.university}.`);
    const attestor = readKey(data, 48);
    if (attestor !== intent.attestor) refuse(`the attestor is ${attestor}, not ${intent.attestor}.`);
    const target = readU64(data, 8 + 8 + 32 + 32);
    if (target !== intent.target) refuse(`the target is ${target} micro-USDC, not ${intent.target}.`);
  }
}
