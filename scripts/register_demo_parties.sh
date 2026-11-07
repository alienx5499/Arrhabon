#!/usr/bin/env bash
# Registers demo universities and visa attestors in the on-chain registry.
# Admin (deployer key) pays the rent. Registered parties need no SOL.
# Prints public keys and ready-to-paste web env lines only. Never prints secrets.
set -euo pipefail
umask 077

cd "$(dirname "$0")/.."
export PATH="/opt/homebrew/opt/rustup/bin:/opt/homebrew/bin:$HOME/.local/share/solana/install/active_release/bin:$PATH"

KEYS=.keys
mkdir -p "$KEYS"
chmod 700 "$KEYS"

say() { printf '==> %s\n' "$*"; }
pubkey() { solana-keygen pubkey "$1"; }

ensure_key() {
  local slug="$1"
  local f="$KEYS/$slug.json"
  if [ ! -f "$f" ]; then
    solana-keygen new --no-bip39-passphrase --silent -o "$f" >/dev/null
  fi
  chmod 600 "$f"
  pubkey "$f"
}

# Fixed party lists: "Name:slug"
UNIVERSITIES=(
  "Seoul Demo University (demo):university"
  "Hanbit University (demo):hanbit-university"
  "Northgate College (demo):northgate-college"
  "Lakeview Institute (demo):lakeview-institute"
)

ATTESTORS=(
  "Demo visa attestor (demo):attestor"
  "Admissions office attestor (demo):admissions-attestor"
  "Visa agent attestor (demo):visa-agent-attestor"
)

say "registering demo universities"
uni_entries=()
names_entries=()

for item in "${UNIVERSITIES[@]}"; do
  name="${item%%:*}"
  slug="${item##*:}"
  pk="$(ensure_key "$slug")"
  cargo run -q -p arrhabon-solana-adapter --bin devnet -- register university "$pk"
  uni_entries+=("$name=$pk")
  names_entries+=("\"$pk\":\"$name\"")
done

say "registering demo attestors"
att_entries=()

for item in "${ATTESTORS[@]}"; do
  name="${item%%:*}"
  slug="${item##*:}"
  pk="$(ensure_key "$slug")"
  cargo run -q -p arrhabon-solana-adapter --bin devnet -- register attestor "$pk"
  att_entries+=("$name=$pk")
  names_entries+=("\"$pk\":\"$name\"")
done

# Format comma-separated lists and JSON map
IFS=','
UNIS_STR="${uni_entries[*]}"
ATTS_STR="${att_entries[*]}"
NAMES_JSON="{$(IFS=','; echo "${names_entries[*]}")}"
unset IFS

echo ""
say "paste these into Vercel env or web/.env.local:"
echo "NEXT_PUBLIC_UNIVERSITIES=$UNIS_STR"
echo "NEXT_PUBLIC_ATTESTORS=$ATTS_STR"
echo "NEXT_PUBLIC_REGISTRY_NAMES=$NAMES_JSON"

