#!/usr/bin/env bash
# Sends test USDC from the deployer's treasury to any wallet (e.g. a Privy embedded wallet).
# Usage: scripts/fund_usdc.sh <localnet|devnet> <wallet-pubkey> <amount> [sol-amount]
set -euo pipefail
umask 077
cd "$(dirname "$0")/.."

cluster="${1:?cluster: localnet or devnet}"
owner="${2:?wallet pubkey}"
amount="${3:?USDC amount}"
sol_amount="${4:-0}"

case "$cluster" in
  localnet) url="http://127.0.0.1:8899" ;;
  devnet) url="https://api.devnet.solana.com" ;;
  *) echo "cluster must be localnet or devnet" >&2; exit 1 ;;
esac

deploy_file="deploy/$cluster.json"
[ -f "$deploy_file" ] || { echo "missing $deploy_file; run the $cluster setup script first" >&2; exit 1; }
mint="$(python3 -c "import json,sys;print(json.load(open(sys.argv[1]))['usdc_mint'])" "$deploy_file")"
payer=".keys/deployer.json"

spl-token --url "$url" --fee-payer "$payer" \
  transfer --owner "$payer" "$mint" "$amount" "$owner" --fund-recipient --allow-unfunded-recipient >/dev/null
echo "sent $amount test USDC to $owner"

if [ "$sol_amount" != "0" ]; then
  solana --url "$url" --keypair "$payer" transfer --allow-unfunded-recipient "$owner" "$sol_amount" >/dev/null
  echo "sent $sol_amount SOL to $owner"
fi
