# Deploying Arrhabon (devnet)

The API runs on Render as a Docker web service (`render.yaml`). The web app runs on Vercel (no `vercel.json` needed: Next.js is detected). Both point at devnet.

Never commit a key file, and never paste a key into an environment variable. Keys go to Render only as secret files.

## 1. Devnet setup (once, after the deployer is funded)

```sh
anchor build
scripts/devnet_setup.sh
```

If it stops with "The devnet faucet refused", fund the printed deployer address at https://faucet.solana.com (cluster: devnet) and rerun. When it finishes, `deploy/devnet.json` holds the public addresses (program, mint, university, attestor) used below.

Give the crank key devnet SOL too; it pays the fees for `expire` and `refund`:

```sh
solana -C .keys/cli-config.yml balance "$(solana-keygen pubkey .keys/crank.json)"
```

## 2. Render: API

1. Render dashboard > New > Blueprint, connect the repository. Render reads `render.yaml` and proposes the `arrhabon-api` service on the Starter plan. Keep the paid plan: the free tier sleeps and the refund crank would stop.
2. It asks for the `sync: false` values. `SOLANA_RPC_URL`, `AURORA_API_KEY` and `ONECLICK_JWT` are optional; leave them blank unless you have them.
3. Before the first deploy, open the service > Environment > Secret Files and add:
   - `crank.json`: contents of `.keys/crank.json`
   - `attestor.json`: contents of `.keys/attestor.json` (only needed while the demo attestor is on; you can add it later)

   Render mounts them at `/etc/secrets/crank.json` and `/etc/secrets/attestor.json`, which is where `CRANK_KEYPAIR` and `DEMO_ATTESTOR_KEYPAIR` point. Paste the file contents in the dashboard; do not copy the files into the repository.
4. Deploy. Note the service URL, e.g. `https://arrhabon-api.onrender.com`.
5. If the logs show `CRANK_KEYPAIR: cannot read keypair file`, the secret file name does not match or the container user (uid 10001) cannot read it; check the name first.

## 3. Vercel: web app

1. Vercel > Add New > Project, import the repository.
2. Root Directory: `web`. Framework Preset: Next.js. Build and install commands: defaults (pnpm is picked up from `pnpm-lock.yaml`). If the install step rejects the pnpm version, add the env var `ENABLE_EXPERIMENTAL_COREPACK=1` so the `packageManager` version in `web/package.json` is used.
3. Environment variables (Production):

   | Variable | Value |
   |---|---|
   | `NEXT_PUBLIC_PRIVY_APP_ID` | Your Privy app id |
   | `NEXT_PUBLIC_API_URL` | The Render service URL, no trailing slash |
   | `NEXT_PUBLIC_SOLANA_RPC_URL` | `https://api.devnet.solana.com` (public; every `NEXT_PUBLIC_*` value ships to browsers, so no private RPC key here) |
   | `NEXT_PUBLIC_PROGRAM_ID` | Program id from `deploy/devnet.json` |
   | `NEXT_PUBLIC_UNIVERSITIES` | Optional, `Name=address` from `deploy/devnet.json` |
   | `NEXT_PUBLIC_ATTESTORS` | Optional, same format |

4. Deploy and note the production URL, e.g. `https://arrhabon.vercel.app`.

## 4. Connect the two

1. Render > `arrhabon-api` > Environment: set `WEB_ORIGIN` to the exact Vercel URL (`https://...vercel.app`, no trailing slash, no path). Save; Render redeploys.
2. Privy dashboard > your app > allowed origins (domains): add the Vercel URL. Keep `http://localhost:3000` only if you still develop locally.

## Test token

The mint is **Arrhabon Test Dollar** (`tUSD`, 6 decimals), with a 10 billion tUSD treasury held by the deployer. Fund a wallet with `scripts/fund_usdc.sh devnet <wallet-pubkey> <amount> [sol-amount]`. After the Vercel deploy, give the mint its name and image (Vercel URL + `/token.json`):

```sh
TOKEN_METADATA_URI=https://<vercel-url>/token.json scripts/token_metadata.sh devnet
```

This rewrites `web/public/token.json` with an absolute image URL (`https://<vercel-url>/token.png`); redeploy the web app so the file is served, since wallets fetch it from there. Rerunning updates the metadata if the URI changed and skips if it is current.

## Test faucet (for judges)

`FAUCET=true` turns on `POST /demo/faucet` and the **Get test funds** button in the web header. It tops the signed-in wallet up to 0.05 devnet SOL (fees) and 1,000 tUSD (contributions), sending only the shortfall, so a funded wallet gets nothing and a second click within 10 minutes is refused.

Judges can claim test funds for any wallet address without logging in at https://arrhabon.vercel.app/faucet.

1. Make a faucet keypair: `solana-keygen new --no-bip39-passphrase -o .keys/faucet.json`.
2. Fund it once from the deployer: `scripts/fund_usdc.sh devnet $(solana-keygen pubkey .keys/faucet.json) 1000000 10` (1M tUSD and 10 SOL, enough for hundreds of judges).
3. On Render, add `faucet.json` as a secret file, set `FAUCET=true` and `FAUCET_KEYPAIR=/etc/secrets/faucet.json`, save and wait for the redeploy. The log line "DEMO faucet ready" prints the faucet address.

If the faucet runs dry, judges can still get devnet SOL at https://faucet.solana.com (cluster: devnet).

## 5. Smoke test

- [ ] `curl https://<render-url>/health` returns `"cluster":"devnet"`, `"chain":"ready"`, `"crank":true`, `"demo_attestor":true`. Right after a deploy `chain` may read `not_initialised` for a few seconds; retry.
- [ ] `curl https://<render-url>/registry` lists the registered university and attestor.
- [ ] Open the Vercel URL, log in with Privy, create a vault, and open its vault page. The browser console shows no CORS errors.
- [ ] Render logs show the crank running every 10 s without errors.

## 6. Demo attestor (on for the devnet demo)

`DEMO_ATTESTOR=true` lets anyone call `POST /demo/attest` and confirm or deny any vault that names the demo attestor. Turn it on only for the recording:

1. Make sure the `attestor.json` secret file is present.
2. Set `DEMO_ATTESTOR=true`, save, wait for the redeploy; `/health` shows `"demo_attestor":true`.
3. Record.

## 7. Turn off / roll back

- **Before real money, demo attestor off:** set `DEMO_ATTESTOR=false`, save, then delete the `attestor.json` secret file. Confirm `/health` shows `"demo_attestor":false`.
- **Bad deploy:** Render > `arrhabon-api` > Events, pick the last good deploy > Rollback. Vercel > Deployments, pick the last good one > Promote to Production.
- **Leaked key:** remove the secret file, stop the service, and replace the key. A leaked crank key only risks its devnet SOL; a leaked attestor key can decide visa outcomes for vaults naming it, so deactivate it with the deployer (registry admin) key, then register a new attestor:

  ```sh
  cargo run -q -p arrhabon-solana-adapter --bin devnet -- registry attestor <attestor-pubkey> --active=false
  ```

  It prints the public keys and the transaction signature. `--active=true` switches it back on, and `registry university <pubkey> --active=false|true` does the same for a university. It runs only against devnet or localnet (`ARRHABON_CLUSTER`).
