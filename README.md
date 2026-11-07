# Arrhabon

Arrhabon (ἀρραβών) is the ancient Greek word for earnest money: a deposit paid to secure an agreement, returned or forfeited by fixed rules.

Arrhabon holds an international student's tuition deposit in a Solana program. The university is paid once the student's visa is issued. If the visa is denied, or nobody confirms the outcome by the deadline, every person who paid in gets back exactly what they paid.

## The problem

Universities take tuition deposits before the visa decision, and when the visa is refused the refund is slow or never comes. In Korea, about 400 + about 170 students at two colleges were left waiting on refunds after visa denials, which pushed the education ministry to write the refund into its rules. In the UK, tens of thousands of student visa applications are refused or withdrawn each year, and the deposit sits with the university either way. A rule in law still depends on the university choosing to pay; Arrhabon makes the rule the custody itself.

| Outlet | Date | What it reports | Link |
|---|---|---|---|
| Korea Herald | 7 Oct 2026 | About 400 foreign students at Singyeongju University owed 2.8bn won ($2.07M) combined; about 170 at Woongji College about 900M won; 115 complaints over five years | https://www.koreaherald.com/article/10895948 |
| ICEF Monitor | 10 Sep 2026 | Korea's education ministry now requires "refunding the tuition paid by students whose visa is denied"; some junior colleges delayed refunds of up to 1bn won ($745k) | https://monitor.icef.com/2026/09/korean-government-introduces-new-quality-controls-for-universities-and-agents/ |
| The PIE News | 5 Oct 2026 | UK, year to June 2026: 19,608 student visas refused and 11,500 applications withdrawn; Pakistan's Q1 refusal rate 39.3% | https://thepienews.com/in-focus-how-uk-unis-are-staying-under-the-5-bca-threshold/ |

## What Arrhabon does

1. A student opens a Vault: a registered university, a registered attestor, a target amount in USDC and a deadline.
2. The student and family members anywhere contribute. Each funder's money is recorded separately.
3. Visa issued: the attestor confirms and the whole vault goes to the university in one transaction.
4. Visa denied, or the deadline passes with no decision: the vault switches to refunding, and anyone (our crank, the funder, a stranger) can trigger each funder's refund.

The program also has `cancel` (the student withdraws a vault before anyone else has contributed); this version does not expose it in the API or the web app.

Three views in the web app: student (create a vault, share the link), funder (contribute, see your share and refund status), university (proof that funds are held and the release condition).

## How Solana enforces it

- **Custody in a PDA.** USDC sits in a token account owned by the vault's program-derived address. No private key can move it; only the program's instructions can.
- **Two exits only.** Money leaves the vault to the university's token account after `confirm_visa`, or to recorded funders after the vault is `Refunding`. There is no instruction that sends it anywhere else, including to the operator.
- **Release only after attestation.** `confirm_visa` must be signed by the vault's attestor, the vault must be open and fully funded, and it fails after the deadline.
- **Permissionless expiry and refund.** `expire` and `refund` need no special signer. If our backend disappears, a funder can still call them. Each funder has one `Contribution` account, so refunds never loop over an unbounded list, and a refunded contribution cannot be refunded twice.
- **Registry.** Universities and attestors must be registered and active in an on-chain registry. A student cannot name themselves (or a friend) as attestor, and the student, university and attestor must be three distinct parties.
- **Legacy SPL Token only.** The mint is fixed in the program's Config and must be a 6-decimal legacy SPL Token mint; Token-2022 mints (with their transfer hooks and fees) are rejected.
- **Deadline cap.** A deadline can be at most 365 days ahead, so money cannot be parked forever.
- **1 USDC minimum** per contribution, so nobody can spam a vault with dust contributions.

The rules live once, in the pure `domain` crate. The on-chain program calls the same transitions the off-chain code uses, and LiteSVM tests with clock warp check the money paths on the compiled program.

## Trust model

- **The attestor is the one trust point.** Someone has to say whether the visa was issued, and the program cannot check that by itself. A dishonest attestor could release a vault for a student whose visa was refused (only if fully funded and before the deadline), or deny a visa that was issued (the money then goes back to the funders, never to the attestor). In the demo the attestor is a clearly labelled demo key. The production path is the university's admissions office together with the student as a multisig, or a licensed agent.
- **Never enable `DEMO_ATTESTOR=true` on a public deployment.** It lets anyone call `POST /demo/attest` and confirm or deny any vault that names the demo attestor. The demo deployment turns it on only for the recording. A production attestor is a separate registered party that signs with its own key, outside this API.
- **Registry admin = program upgrade authority.** Only the program's upgrade authority can initialise Config, and it becomes the admin that registers and deactivates universities and attestors. Today that is one project key. The admin cannot move money in a vault; it can stop new vaults from naming a deactivated party.
- **The program is upgradeable for now.** An upgrade authority can replace the program code, so for now you are also trusting whoever holds that key. Freezing the upgrade authority (or moving it to a multisig) is the step before real money.
- **Devnet only.** The deployed program runs on devnet with a test USDC mint. Nothing here has been audited.

## Architecture

Domain-driven design. The core domain is Escrow (vault lifecycle, custody, release, refund) and owns every money rule. Funding (getting money in, including cross-chain through Aurora) never decides where money goes out. Attestation decides who may confirm a visa outcome. Views are read models for the three screens.

Dependencies point inward. `api` depends on `app`, `app` depends on `domain`, and the adapters implement ports defined in `app`. The on-chain program depends on `domain` directly and adapts accounts to it. `domain` has no I/O, no async and no Solana types, so it compiles both for the host and for the Solana VM.

| Path | Layer | What it holds |
|---|---|---|
| `crates/domain` | Domain | `Vault` aggregate, `Contribution`, `Usdc` (micro-units, checked math), `Deadline`, `VaultStatus` state machine, limits, errors |
| `programs/arrhabon` | Adapter (on chain) | Anchor 1.2.1 program: Config, registry, Vault and Contribution PDAs, the vault token account; each instruction calls a domain transition |
| `crates/app` | Application | Commands (`CreateVault`, `Contribute`, `AttestVisa`, `ExpireDue`, `RunRefunds`, `QuoteCrossChainFunding`), queries (`VaultView`, `FunderView`, `UniversityProof`, `RegistryView`), ports (`VaultLedger`, `AttestorSigner`, `SwapQuoter`, `Clock`), in-memory fakes |
| `crates/solana_adapter` | Adapter | `VaultLedger` over `anchor-client`, chain clock, keypair attestor, relay guard; `devnet` CLI for setup and a scripted run |
| `crates/aurora_adapter` | Adapter | `SwapQuoter` over Aurora's Swap API (or NEAR 1Click) |
| `crates/api` | Composition root | Axum HTTP routes, the refund crank, the demo attestor endpoint; no business logic |
| `web/` | Client | Next.js + Privy embedded Solana wallets; renders views and asks the wallet to sign; no money rules |

The API never holds a user's key. For create and contribute it returns an unsigned transaction; the user's wallet signs it and the API relays it through `/tx/submit`, which only forwards transactions whose instructions call the Arrhabon program (plus optional compute-budget instructions).

## Aurora integration

A relative who holds USDC on another chain (for example Base) can get a quote for funding a vault through Aurora's Swap API (`POST /funding/quote`). The quote shows the minimum USDC that will arrive on Solana (`min_usdc_out`), the fee and the expiry, and it sets `refundTo` to the funder's origin-chain address so a failed swap returns the money where it came from. Swap status maps to `SUCCESS`, `REFUNDED`, `FAILED` or pending (`GET /funding/status`).

- **Dry quotes by default.** Every quote is `dry: true`: price only, no deposit address reserved, nothing to send money to.
- **`AURORA_LIVE` guard.** A live (non-dry) quote that reserves a deposit address is refused unless `AURORA_LIVE=true` is set.
- **Mainnet vs devnet.** Aurora runs on mainnet only, and the vault runs on devnet with a test mint, so the demo shows real quotes but does not settle a swap into a devnet vault. The production path is a swap that pays into the funder's Solana wallet, which then contributes.
- **Stretch: Intents Connect.** A deposit from any chain that triggers `contribute` directly, and a persistent any-chain deposit address per vault. Intents Connect is TypeScript-first, so this would live in the web layer. Not built yet.

With `AURORA_API_KEY` set the adapter calls the Aurora Swap API; without it, it calls the same API through NEAR 1Click.

## Quick start

Prerequisites: Rust (stable), the Solana CLI, Anchor 1.2.1 (`avm install 1.2.1`), Node.js with pnpm. All keys are project keys under `.keys/` (gitignored, mode 600); nothing reads `~/.config/solana/id.json`.

1. **Build the program.**
   ```sh
   anchor build
   ```
   If Homebrew's `cargo` shadows rustup's, run `PATH=/opt/homebrew/opt/rustup/bin:$PATH anchor build`.

2. **Set up devnet.**
   ```sh
   scripts/devnet_setup.sh
   ```
   It creates project keypairs (deployer, crank, attestor, university, student, two funders), airdrops SOL, deploys the program, creates a 6-decimal test USDC mint, mints test USDC to the two funders, initialises Config and registers the university and attestor. It refuses any RPC that is not devnet and is safe to rerun. Public addresses are written to `deploy/devnet.json`.

3. **Faucet step.** The devnet faucet rate-limits airdrops. If the script stops with "The devnet faucet refused", open https://faucet.solana.com (cluster: devnet), request SOL for the deployer address it prints, and rerun the script. Wallets you log in with on the web app also need a little devnet SOL from the same faucet, and test USDC from the project mint:
   ```sh
   MINT=$(solana-keygen pubkey .keys/usdc-mint.json)
   spl-token -C .keys/cli-config.yml create-account "$MINT" --owner <WALLET>
   spl-token -C .keys/cli-config.yml mint "$MINT" 2000 --recipient-owner <WALLET>
   ```

4. **Optional scripted run.** `cargo run -p arrhabon-solana-adapter --bin devnet -- run` drives both paths (release and refund) on devnet from the CLI and prints explorer links.

5. **Run the API.**
   ```sh
   DEMO_ATTESTOR=true WEB_ORIGIN=http://localhost:3000 cargo run -p arrhabon-api
   ```
   It listens on `http://127.0.0.1:8080`; `GET /health` shows whether the chain connection, crank and demo attestor are on. Add `SOLANA_RPC_URL` for a private devnet RPC (Helius); the URL is kept out of logs.

6. **Run the web app.**
   ```sh
   cd web
   pnpm install
   NEXT_PUBLIC_PRIVY_APP_ID=<your Privy app id> NEXT_PUBLIC_API_URL=http://localhost:8080 pnpm dev
   ```
   Open http://localhost:3000. The Privy app id comes from your own Privy dashboard (enable Solana embedded wallets).

### Run locally (no faucet)

A local `solana-test-validator` replaces devnet when the faucet is rate-limited:

```sh
anchor build
scripts/localnet_setup.sh            # validator in .localnet/, airdrops, deploy, test USDC, config, registry
ARRHABON_CLUSTER=localnet cargo run -p arrhabon-solana-adapter --bin devnet -- run
ARRHABON_CLUSTER=localnet CRANK_INTERVAL_SECS=5 DEMO_ATTESTOR=true cargo run -p arrhabon-api
scripts/localnet_setup.sh --stop     # stop the validator; the ledger is kept for the next start
```

`ARRHABON_CLUSTER=localnet` is accepted only when `SOLANA_RPC_URL` is on `127.0.0.1` or `localhost` (default `http://127.0.0.1:8899`); a mainnet genesis is refused on every cluster. Addresses go to `deploy/localnet.json`. To drive the API by hand, sign its unsigned transactions with a project key: `echo "$UNSIGNED" | base64 -d | target/debug/devnet sign student | base64`, then `POST /tx/submit`. For the web app, point `NEXT_PUBLIC_SOLANA_RPC_URL` at `http://127.0.0.1:8899`; explorer links on localnet need the explorer's custom-RPC setting.

### Test token

The demo mint is a 6-decimal test token named **Arrhabon Test Dollar** (`tUSD`), not USDC and worth nothing. Each setup script mints a 10 billion tUSD treasury to the deployer. Send some to any wallet (e.g. a Privy embedded wallet):

```sh
scripts/fund_usdc.sh <localnet|devnet> <wallet-pubkey> <amount> [sol-amount]
```

Name and image come from Metaplex Token Metadata pointing at `web/public/token.json` (image `web/public/token.png`). Localnet uses `http://localhost:3000/token.json`. On devnet set `TOKEN_METADATA_URI` to the deployed web app, e.g. `TOKEN_METADATA_URI=https://<app>.vercel.app/token.json scripts/devnet_setup.sh` (or `scripts/token_metadata.sh devnet` on its own); without it the metadata step is skipped.

### Docker (API)

```sh
docker build -t arrhabon-api .
docker run --rm -p 8080:8080 \
  -v "$PWD/.keys:/app/.keys:ro" \
  -e DEMO_ATTESTOR=true -e WEB_ORIGIN=http://localhost:3000 \
  arrhabon-api
```

The image contains no keys. The container runs as a non-root user (uid 10001) with `HOST=0.0.0.0`, and reads `.keys/crank.json` and `.keys/attestor.json` relative to `/app`. On Linux the mounted key files must be readable by that user: either add `--user "$(id -u)"` or mount copies owned by uid 10001. On a hosting platform, mount the keys as secret files and point `CRANK_KEYPAIR` / `DEMO_ATTESTOR_KEYPAIR` at them.

## Environment variables

API (`crates/api`):

| Variable | Default | Meaning |
|---|---|---|
| `ARRHABON_CLUSTER` | `devnet` | `devnet` (genesis-hash checked) or `localnet` (RPC host must be `127.0.0.1` or `localhost`). Mainnet is always refused. Also read by the `devnet` CLI. |
| `SOLANA_RPC_URL` | per cluster | `https://api.devnet.solana.com` or `http://127.0.0.1:8899`. A cluster the guard refuses stops the server. Treated as a secret in logs. |
| `HOST` | `127.0.0.1` | Bind address. Use `0.0.0.0` in containers (the Dockerfile sets it). |
| `PORT` | `8080` | Listen port. |
| `WEB_ORIGIN` | `http://localhost:3000` | The one browser origin allowed by CORS. Exact origin, no wildcard, no trailing slash. |
| `CRANK` | `true` | Run the refund crank (expire due vaults, refund funders). |
| `CRANK_INTERVAL_SECS` | `10` | Seconds between crank runs; minimum 5. |
| `CRANK_KEYPAIR` | `.keys/crank.json` | Key that pays fees for `expire` and `refund`. Needs devnet SOL. |
| `DEMO_ATTESTOR` | `false` | Enable `POST /demo/attest`, which signs visa outcomes with a demo key. Recording only; never on a public deployment (see Trust model). |
| `DEMO_ATTESTOR_KEYPAIR` | `.keys/attestor.json` | The demo attestor key; must be a registered attestor. |
| `AURORA_API_KEY` | unset | Aurora Swap API key. Unset: NEAR 1Click is used. |
| `ONECLICK_JWT` | unset | Optional 1Click partner token (without it 1Click adds 25 bps). |
| `AURORA_BASE_URL` | per API | Override the swap API base URL. |
| `AURORA_TIMEOUT_SECS` | `15` | Swap API request timeout. |
| `AURORA_LIVE` | `false` | Allow live quotes that reserve a deposit address. Leave off for the demo. |
| `RUST_LOG` | `info` | Log filter. |

Web (`web/`):

| Variable | Default | Meaning |
|---|---|---|
| `NEXT_PUBLIC_PRIVY_APP_ID` | required | Your Privy app id. |
| `NEXT_PUBLIC_API_URL` | `http://localhost:8080` | The API's base URL. |
| `NEXT_PUBLIC_SOLANA_RPC_URL` | `https://api.devnet.solana.com` | RPC the browser uses. |
| `NEXT_PUBLIC_SITE_URL` | the Vercel URL, else `http://localhost:3000` | Public site URL; makes share-image (og:image, twitter:image) links absolute. |
| `NEXT_PUBLIC_PROGRAM_ID` | the deployed program | Arrhabon program id, used to check transactions before signing. |
| `NEXT_PUBLIC_UNIVERSITIES` | empty | Display names, `Name=address,Name2=address2`. Also the picker fallback when `/registry` is unreachable. |
| `NEXT_PUBLIC_ATTESTORS` | empty | Same format, for attestors. |
| `NEXT_PUBLIC_REGISTRY_NAMES` | empty | JSON map of address to display name, e.g. `{"<address>":"Uni A"}`. Checked first; unnamed addresses show shortened. |

## Testing

```sh
anchor build            # program tests load target/deploy/arrhabon.so
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cd web && pnpm test
```

`cargo test --workspace` runs 218 tests:

| Crate | Tests |
|---|---|
| `arrhabon-domain` | 68 (unit and property tests of every invariant) |
| `arrhabon` (program, LiteSVM with clock warp) | 46 |
| `arrhabon-app` | 29 |
| `arrhabon-api` | 27 |
| `arrhabon-solana-adapter` | 24 |
| `arrhabon-aurora-adapter` | 14, plus 1 ignored live dry-quote test (`cargo test -p arrhabon-aurora-adapter -- --ignored`) |

## Regulation and tax notes

- Rules on holding and sending crypto, including stablecoins, differ by country. Some students' home countries restrict crypto or foreign exchange. Check your own country's rules before using anything like this with real money.
- Funders do not have to be in the student's country. Arrhabon is built for family abroad paying in for a student.
- Paying tuition, receiving a refund or swapping between chains may have tax consequences where you live.
- This is a demo on devnet. Nothing here is financial, legal or tax advice.

## License

MIT
