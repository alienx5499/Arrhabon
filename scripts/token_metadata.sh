#!/usr/bin/env bash
# Names the test mint "Arrhabon Test Dollar" (tUSD) via Metaplex Token Metadata.
# Usage: TOKEN_METADATA_URI=<url of token.json> scripts/token_metadata.sh <localnet|devnet>
# Writes web/public/token.json with image = <dirname of the URI>/token.png, then creates or
# updates the on-chain metadata account. Devnet without TOKEN_METADATA_URI: prints a note, skips.
set -euo pipefail
cd "$(dirname "$0")/.."

cluster="${1:?cluster: localnet or devnet}"
case "$cluster" in
  localnet) rpc="http://127.0.0.1:8899"; default_uri="http://localhost:3000/token.json" ;;
  devnet) rpc="${SOLANA_RPC_URL:-https://api.devnet.solana.com}"; default_uri="" ;;
  *) echo "cluster must be localnet or devnet" >&2; exit 1 ;;
esac

uri="${TOKEN_METADATA_URI:-$default_uri}"
if [ -z "$uri" ]; then
  cat <<EOF
==> note: TOKEN_METADATA_URI not set; skipping token metadata.
    Deploy web/ first, then rerun with e.g.
    TOKEN_METADATA_URI=https://<your-app>.vercel.app/token.json scripts/devnet_setup.sh
EOF
  exit 0
fi

cat >web/public/token.json <<EOF
{
  "name": "Arrhabon Test Dollar",
  "symbol": "tUSD",
  "description": "Test token for the Arrhabon devnet demo. Not USDC and has no value.",
  "image": "${uri%/*}/token.png"
}
EOF
chmod 644 web/public/token.json

printf '==> token metadata uri %s\n' "$uri"
ARRHABON_CLUSTER="$cluster" SOLANA_RPC_URL="$rpc" \
  cargo run -q -p arrhabon-solana-adapter --bin devnet -- token-metadata --uri "$uri"
