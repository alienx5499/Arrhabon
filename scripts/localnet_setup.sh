#!/usr/bin/env bash
# Idempotent localnet setup (no faucet): a background solana-test-validator under .localnet/,
# SOL airdrops to the project keypairs, program deploy, test USDC, its token metadata, config and
# registry. The Metaplex Token Metadata program is dumped from devnet once (read-only) into
# .localnet/ and loaded at genesis; a ledger created before that lacks it (reset: rm -rf .localnet/ledger).
# `--stop` stops the validator (the ledger is kept, so a restart resumes the same state).
# Prints public keys only. Never touches ~/.config/solana/id.json.
set -euo pipefail
umask 077

cd "$(dirname "$0")/.."
# Homebrew solana (4.x) first: the 3.x CLI cannot parse the SBPF binary anchor 1.2 builds.
export PATH="/opt/homebrew/opt/rustup/bin:/opt/homebrew/bin:$HOME/.local/share/solana/install/active_release/bin:$PATH"

RPC="http://127.0.0.1:8899"
DIR=.localnet
PID="$DIR/validator.pid"
LOG="$DIR/validator.log"
CFG="$DIR/cli-config.yml"
KEYS=.keys
SO=target/deploy/arrhabon.so
PROGRAM_KEY="$KEYS/arrhabon-program.json"
MINT_KEY="$KEYS/usdc-mint.json"
PARTIES=(deployer crank attestor university student funder1 funder2)
FUNDER_USDC=10000
METAPLEX=metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s
METAPLEX_SO="$DIR/mpl_token_metadata.so"

say() { printf '==> %s\n' "$*"; }
die() { printf 'error: %s\n' "$*" >&2; exit 1; }
pubkey() { solana-keygen pubkey "$1"; }
sol() { solana -C "$CFG" "$@"; }
balance() { sol balance "$1" | awk '{print $1}'; }
lt() { awk -v a="$1" -v b="$2" 'BEGIN{exit !(a<b)}'; }
healthy() { curl -sf -X POST -H 'Content-Type: application/json' \
  -d '{"jsonrpc":"2.0","id":1,"method":"getHealth"}' "$RPC" 2>/dev/null | grep -q '"ok"'; }
running() { [ -f "$PID" ] && kill -0 "$(cat "$PID")" 2>/dev/null; }

if [ "${1:-}" = "--stop" ]; then
  if running; then
    kill "$(cat "$PID")"
    say "validator stopped"
  else
    say "validator not running"
  fi
  rm -f "$PID"
  exit 0
fi

[ -f "$SO" ] || die "$SO missing: run 'anchor build' first"
[ -f "$PROGRAM_KEY" ] || die "$PROGRAM_KEY missing (the program keypair)"
DECLARED_ID="$(sed -n 's/^declare_id!("\([^"]*\)");/\1/p' programs/arrhabon/src/lib.rs)"
PROGRAM_ID="$(pubkey "$PROGRAM_KEY")"
[ "$PROGRAM_ID" = "$DECLARED_ID" ] || die "$PROGRAM_KEY is $PROGRAM_ID but declare_id! is $DECLARED_ID"

mkdir -p "$DIR" "$KEYS" deploy
chmod 700 "$KEYS"
for p in "${PARTIES[@]}"; do
  [ -f "$KEYS/$p.json" ] || solana-keygen new --no-bip39-passphrase --silent -o "$KEYS/$p.json" >/dev/null
done
[ -f "$MINT_KEY" ] || solana-keygen new --no-bip39-passphrase --silent -o "$MINT_KEY" >/dev/null
chmod 600 "$KEYS"/*.json

# Localnet-only CLI config; the devnet one in .keys/ is left alone.
cat >"$CFG" <<EOF
json_rpc_url: "$RPC"
websocket_url: ""
keypair_path: "$PWD/$KEYS/deployer.json"
address_labels: {}
commitment: confirmed
EOF

if healthy; then
  say "validator already up at $RPC"
else
  running && die "validator process $(cat "$PID") is up but unhealthy; see $LOG or run $0 --stop"
  if [ ! -s "$METAPLEX_SO" ]; then
    say "fetching the Metaplex Token Metadata program from devnet (cached in $METAPLEX_SO)"
    solana program dump --url https://api.devnet.solana.com "$METAPLEX" "$METAPLEX_SO" >/dev/null
  fi
  say "starting solana-test-validator (log: $LOG)"
  nohup solana-test-validator --ledger "$DIR/ledger" --bind-address 127.0.0.1 \
    --rpc-port 8899 --bpf-program "$METAPLEX" "$METAPLEX_SO" --quiet >"$LOG" 2>&1 &
  echo $! >"$PID"
  for _ in $(seq 60); do healthy && break; sleep 1; done
  healthy || die "validator not healthy after 60 s; see $LOG"
fi
say "genesis $(solana genesis-hash --url "$RPC")"

say "airdrops"
for p in "${PARTIES[@]}"; do
  k="$(pubkey "$KEYS/$p.json")"
  want=10; [ "$p" = deployer ] && want=100
  lt "$(balance "$k")" "$((want / 2))" && sol airdrop "$want" "$k" >/dev/null
  printf '    %-10s %s %s SOL\n' "$p" "$k" "$(balance "$k")"
done

say "program $PROGRAM_ID"
needs_deploy=1
if sol program show "$PROGRAM_ID" >/dev/null 2>&1; then
  tmp="$(mktemp)"
  sol program dump "$PROGRAM_ID" "$tmp" >/dev/null
  cmp -s -n "$(wc -c <"$SO" | tr -d ' ')" "$SO" "$tmp" && needs_deploy=0
  rm -f "$tmp"
fi
if [ "$needs_deploy" = 1 ]; then
  # Filtered: on a retried buffer write the CLI prints the buffer's recovery seed phrase.
  sol program deploy "$SO" --program-id "$PROGRAM_KEY" --upgrade-authority "$KEYS/deployer.json" \
    2>&1 | grep -E '^(Program Id|Signature|Error)'
else
  say "deployed binary is current"
fi

MINT="$(pubkey "$MINT_KEY")"
say "test USDC mint $MINT (6 decimals)"
sol account "$MINT" >/dev/null 2>&1 || spl-token -C "$CFG" create-token --decimals 6 "$MINT_KEY" >/dev/null
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
  spl-token -C "$CFG" balance "$MINT" --owner "$k" >/dev/null 2>&1 ||
    spl-token -C "$CFG" create-account "$MINT" --owner "$k" --fee-payer "$KEYS/deployer.json" >/dev/null
  lt "$(spl-token -C "$CFG" balance "$MINT" --owner "$k")" 1000 &&
    spl-token -C "$CFG" mint "$MINT" "$FUNDER_USDC" --recipient-owner "$k" >/dev/null
  say "$p holds $(spl-token -C "$CFG" balance "$MINT" --owner "$k") USDC"
done

if sol account "$METAPLEX" >/dev/null 2>&1; then
  scripts/token_metadata.sh localnet
else
  say "note: this ledger predates the Metaplex program; skipping token metadata"
  say "      to add it: $0 --stop && rm -rf $DIR/ledger && $0"
fi

say "config and registry"
ARRHABON_CLUSTER=localnet SOLANA_RPC_URL="$RPC" \
  cargo run -q -p arrhabon-solana-adapter --bin devnet -- setup
say "done; addresses in deploy/localnet.json; stop with: $0 --stop"
