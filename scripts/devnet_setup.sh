#!/usr/bin/env bash
# Idempotent devnet setup: project keypairs, SOL, program deploy, test USDC, its token metadata
# (needs TOKEN_METADATA_URI, else skipped), config and registry.
# Prints public keys only. Never touches ~/.config/solana/id.json.
set -euo pipefail
umask 077 # key files are never world-readable, not even briefly

cd "$(dirname "$0")/.."
# Homebrew solana (4.x) first: the 3.x CLI cannot parse the SBPF binary anchor 1.2 builds.
export PATH="/opt/homebrew/opt/rustup/bin:/opt/homebrew/bin:$HOME/.local/share/solana/install/active_release/bin:$PATH"

RPC="${SOLANA_RPC_URL:-https://api.devnet.solana.com}"
DEVNET_GENESIS="EtWTRABZaYq6iMfeYKouRu166VU2xqa1wcaWoxPkrZBG"
KEYS=.keys
CFG="$KEYS/cli-config.yml"
SO=target/deploy/arrhabon.so
PROGRAM_KEY="$KEYS/arrhabon-program.json"
MINT_KEY="$KEYS/usdc-mint.json"
PARTIES=(deployer crank attestor university student funder1 funder2)
DEPLOYER_MIN_SOL=4      # deploy/upgrade (~3 SOL buffer + fees) plus funding the parties
DEPLOYER_IDLE_SOL=1.5   # program already current: only party funding and the mint
PARTY_SOL=0.2           # fees and rent for vault, contribution and token accounts
FUNDER_USDC=10000

say() { printf '==> %s\n' "$*"; }
die() { printf 'error: %s\n' "$*" >&2; exit 1; }
pubkey() { solana-keygen pubkey "$1"; }
sol() { solana -C "$CFG" "$@"; }
balance() { sol balance "$1" | awk '{print $1}'; }
lt() { awk -v a="$1" -v b="$2" 'BEGIN{exit !(a<b)}'; }

[ -f "$SO" ] || die "$SO missing: run 'anchor build' first"
[ -f "$PROGRAM_KEY" ] || die "$PROGRAM_KEY missing (the program keypair)"

DECLARED_ID="$(sed -n 's/^declare_id!("\([^"]*\)");/\1/p' programs/arrhabon/src/lib.rs)"
ANCHOR_IDS="$(sed -n 's/^arrhabon = "\([^"]*\)"/\1/p' Anchor.toml | sort -u)"
KEY_ID="$(pubkey "$PROGRAM_KEY")"
[ -n "$DECLARED_ID" ] || die "no declare_id! found in programs/arrhabon/src/lib.rs"
[ "$KEY_ID" = "$DECLARED_ID" ] || die "$PROGRAM_KEY is $KEY_ID but declare_id! is $DECLARED_ID; fix the keypair or run 'anchor keys sync' and rebuild"
[ "$ANCHOR_IDS" = "$DECLARED_ID" ] || die "Anchor.toml program ids ($(echo "$ANCHOR_IDS" | tr '\n' ' ')) differ from declare_id! $DECLARED_ID"

genesis="$(solana genesis-hash --url "$RPC")"
[ "$genesis" = "$DEVNET_GENESIS" ] || die "$RPC is not devnet (genesis $genesis); refusing"

mkdir -p "$KEYS" deploy
chmod 700 "$KEYS"

say "keypairs in $KEYS/"
for p in "${PARTIES[@]}"; do
  f="$KEYS/$p.json"
  [ -f "$f" ] || solana-keygen new --no-bip39-passphrase --silent -o "$f" >/dev/null
  chmod 600 "$f"
  printf '    %-10s %s\n' "$p" "$(pubkey "$f")"
done
[ -f "$MINT_KEY" ] || solana-keygen new --no-bip39-passphrase --silent -o "$MINT_KEY" >/dev/null
chmod 600 "$MINT_KEY" "$PROGRAM_KEY"

# Project-local CLI config: every solana / spl-token call signs with the deployer.
cat >"$CFG" <<EOF
json_rpc_url: "$RPC"
websocket_url: ""
keypair_path: "$PWD/$KEYS/deployer.json"
address_labels: {}
commitment: confirmed
EOF
chmod 600 "$CFG"

DEPLOYER="$(pubkey "$KEYS/deployer.json")"
PROGRAM_ID="$KEY_ID"
MINT="$(pubkey "$MINT_KEY")"

# Full deploy budget unless the deployed binary already matches $SO (no upgrade needed).
needs_deploy=1
if sol program show "$PROGRAM_ID" >/dev/null 2>&1; then
  tmp="$(mktemp)"
  sol program dump "$PROGRAM_ID" "$tmp" >/dev/null
  cmp -s -n "$(wc -c <"$SO" | tr -d ' ')" "$SO" "$tmp" && needs_deploy=0
  rm -f "$tmp"
fi
if [ "$needs_deploy" = 1 ]; then need=$DEPLOYER_MIN_SOL; else need=$DEPLOYER_IDLE_SOL; fi
for attempt in 1 2 3 4 5; do
  bal="$(balance "$DEPLOYER")"
  lt "$bal" "$need" || break
  say "balance $bal SOL; airdrop attempt $attempt"
  if sol airdrop 2 "$DEPLOYER" >/dev/null 2>&1 || sol airdrop 1 "$DEPLOYER" >/dev/null 2>&1; then
    sleep 2
  else
    sleep $((attempt * 10))
  fi
done
bal="$(balance "$DEPLOYER")"
if lt "$bal" "$need"; then
  cat >&2 <<EOF

The devnet faucet refused (rate limit). Deployer has $bal SOL, needs $need.
Do this, then rerun scripts/devnet_setup.sh:
  1. Open https://faucet.solana.com (cluster: devnet)
  2. Request SOL for: $DEPLOYER
EOF
  exit 2
fi
say "deployer $DEPLOYER has $bal SOL"

say "program $PROGRAM_ID"
if [ "$needs_deploy" = 0 ]; then
  say "deployed binary is current"
else
  say "deploying program (initial deploy or upgrade)"
  sol program deploy "$SO" --program-id "$PROGRAM_KEY" --upgrade-authority "$KEYS/deployer.json"
fi

say "funding parties with SOL"
for p in crank attestor university student funder1 funder2; do
  k="$(pubkey "$KEYS/$p.json")"
  if lt "$(balance "$k")" 0.1; then
    sol transfer --allow-unfunded-recipient "$k" "$PARTY_SOL" >/dev/null
  fi
done

say "test USDC mint $MINT (6 decimals, deployer is mint authority)"
if ! sol account "$MINT" >/dev/null 2>&1; then
  spl-token -C "$CFG" create-token --decimals 6 "$MINT_KEY" >/dev/null
fi
# Treasury: 10 billion test USDC held by the deployer, so testing never runs dry.
TREASURY_USDC=10000000000
DEPLOYER_PK="$(pubkey "$KEYS/deployer.json")"
spl-token -C "$CFG" balance "$MINT" --owner "$DEPLOYER_PK" >/dev/null 2>&1 ||
  spl-token -C "$CFG" create-account "$MINT" --owner "$DEPLOYER_PK" --fee-payer "$KEYS/deployer.json" >/dev/null
if lt "$(spl-token -C "$CFG" balance "$MINT" --owner "$DEPLOYER_PK")" 1000000000; then
  spl-token -C "$CFG" mint "$MINT" "$TREASURY_USDC" --recipient-owner "$DEPLOYER_PK" >/dev/null
fi
say "treasury holds $(spl-token -C "$CFG" balance "$MINT" --owner "$DEPLOYER_PK") USDC"
for p in funder1 funder2; do
  k="$(pubkey "$KEYS/$p.json")"
  if ! spl-token -C "$CFG" balance "$MINT" --owner "$k" >/dev/null 2>&1; then
    spl-token -C "$CFG" create-account "$MINT" --owner "$k" --fee-payer "$KEYS/deployer.json" >/dev/null
  fi
  held="$(spl-token -C "$CFG" balance "$MINT" --owner "$k")"
  if lt "$held" 1000; then
    spl-token -C "$CFG" mint "$MINT" "$FUNDER_USDC" --recipient-owner "$k" >/dev/null
  fi
  say "$p holds $(spl-token -C "$CFG" balance "$MINT" --owner "$k") USDC"
done

SOLANA_RPC_URL="$RPC" scripts/token_metadata.sh devnet

say "config and registry"
SOLANA_RPC_URL="$RPC" cargo run -q -p arrhabon-solana-adapter --bin devnet -- setup
say "done; addresses in deploy/devnet.json"
