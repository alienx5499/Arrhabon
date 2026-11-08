<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="https://github.com/user-attachments/assets/b63ac62e-4127-4214-ad3c-2d6919d33490">
    <img src="https://github.com/user-attachments/assets/00bcbb15-a3fa-4a59-91a3-fb2d85d6c2ad" alt="Arrhabon logo: a gold coin held under a glass bell jar, cut into a navy seal" width="112">
  </picture>
</p>

<h1 align="center">Arrhabon</h1>

<p align="center"><b>Tuition deposits that refund themselves.</b></p>

<p align="center"><a href="https://github.com/alienx5499/Arrhabon/actions/workflows/ci.yml"><img src="https://github.com/alienx5499/Arrhabon/actions/workflows/ci.yml/badge.svg" alt="CI"></a> <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-1f3fbf?style=flat-square" alt="MIT license"></a></p>

![A student with a suitcase stands on a long road leading to a distant stone university gateway, with a gold coin held safely under a glass bell jar and a dashed path looping back to family homes](https://github.com/user-attachments/assets/5250ddf1-3cbd-4608-9864-a89477dfab58)

**Arrhabon holds an international student's tuition deposit in a Solana program. Family members anywhere contribute in USDC, with each person's contribution recorded separately. The university is paid once the student's visa is issued, and if the visa is refused or the deadline passes with no decision, every contributor gets their money back automatically.**

![Rust](https://img.shields.io/badge/Rust-1.89-dea584?style=flat-square&logo=rust&logoColor=white) ![Anchor](https://img.shields.io/badge/Anchor-1.2.1-3b82f6?style=flat-square) ![Solana](https://img.shields.io/badge/Solana-Devnet-14F195?style=flat-square&logo=solana&logoColor=black) ![Next.js](https://img.shields.io/badge/Next.js-16.4-000000?style=flat-square&logo=nextdotjs&logoColor=white) ![React](https://img.shields.io/badge/React-19.3-149ECA?style=flat-square&logo=react&logoColor=white) ![TypeScript](https://img.shields.io/badge/TypeScript-5.9-3178C6?style=flat-square&logo=typescript&logoColor=white)

![Built with Solana, Aurora, Privy, Render, Vercel and Next.js](https://github.com/user-attachments/assets/12731468-f84f-4a39-9044-56de26848b15)

- **Live demo:** https://arrhabon.vercel.app
- **Test funds:** https://arrhabon.vercel.app/faucet
- **GitHub:** https://github.com/alienx5499/Arrhabon
- **API:** https://arrhabon-api.onrender.com
- **Video (2:59):** https://youtu.be/Ae6PteWn6ko
- **Program on Solana Explorer:** https://explorer.solana.com/address/3ipoFfDp56kURD4cYvjhynFz1GfNj94gcyWk7BddTEfd?cluster=devnet

https://github.com/user-attachments/assets/c40e138b-f327-4140-8b88-fedf754071cc

Watch in 4K on YouTube: https://youtu.be/Ae6PteWn6ko

Arrhabon runs on Solana devnet with test USDC. No real money moves, except the one Aurora swap shown in the video.

| At a glance | |
|---|---|
| Money flow | One vault PDA, two exits: university or every funder |
| Refunds | Permissionless; the crank or anyone triggers them |
| Tests | 286 Rust + 64 web passing (`cargo test --workspace` and `pnpm test`) |
| Built on | Solana, Aurora Intents, Privy |

## For judges: 5-minute path

Video: [0:55](https://youtu.be/Ae6PteWn6ko?t=55) log in and test funds &middot; [1:07](https://youtu.be/Ae6PteWn6ko?t=67) open a vault &middot; [1:16](https://youtu.be/Ae6PteWn6ko?t=76) Aurora cross-chain swap &middot; [1:42](https://youtu.be/Ae6PteWn6ko?t=102) family pays in &middot; [1:50](https://youtu.be/Ae6PteWn6ko?t=110) visa issued and university paid &middot; [1:59](https://youtu.be/Ae6PteWn6ko?t=119) visa denied and automatic refund &middot; [2:11](https://youtu.be/Ae6PteWn6ko?t=131) university proof &middot; [2:18](https://youtu.be/Ae6PteWn6ko?t=138) architecture

1. Open the [Live demo](https://arrhabon.vercel.app) and click "Log in" to sign in with an email address (embedded Solana wallet via Privy, no seed phrase needed).
2. Click "Get test funds" in the header or open [/faucet](https://arrhabon.vercel.app/faucet) to claim devnet SOL and test USDC (or enter any wallet address and click "Send test funds").
3. Click "Open a vault" in the navigation bar, choose a registered university and attestor, set a USDC target and a visa deadline, then click "Open vault".
4. On the vault page, enter an amount and click "Contribute USDC", or share the link with family to contribute. Relatives holding tokens on Base, Arbitrum or Ethereum can check rates under "Fund from another chain".
5. Under the demo attestor panel, click "Demo: visa issued" to release funds to the university in one transaction, or "Demo: visa denied" to switch to refunding and return funds to each contributor.

Read the security design in [Trust model](#trust-model). Run `cargo test --workspace` and `cd web && pnpm test` to run the test suite.

## Built for families, not crypto users

- **Email login and embedded wallets.** Students and families log in with an email address. Privy creates an embedded Solana wallet automatically, so nobody manages a seed phrase or a browser extension.
- **Plain words.** The interface says "Open a vault", "Collecting deposits", "Visa issued: paying the university" and "Refunding every funder". No raw addresses in headlines.
- **One-click test funds.** Anyone can get devnet SOL and test USDC at [/faucet](https://arrhabon.vercel.app/faucet) without logging in.
- **Clear progress.** Each relative sees what they paid, what is still needed, and a countdown to the visa deadline.
- **Money never stays locked.** If the visa is refused, or nobody confirms the outcome before the deadline, every contributor is refunded.

## Tools and where they are used

<table>
<tr><th width="210">Tool</th><th>How Arrhabon uses it</th><th>Where</th></tr>
<tr><td width="210" align="center"><picture><source media="(prefers-color-scheme: dark)" srcset="https://github.com/user-attachments/assets/a0e053f9-e78e-430a-b464-0d7139ad8426"><img src="https://github.com/user-attachments/assets/bc1c4273-9fa9-402b-8708-331cc53f7969" alt="Solana" width="170"></picture></td><td>Anchor 1.2.1 program on Solana devnet. Vault and Contribution program-derived addresses (PDAs) custody the funds. Fixed 6-decimal legacy SPL token mint. Permissionless expiry and refunds. Registry for universities and attestors.</td><td><a href="programs/arrhabon/"><code>programs/arrhabon/</code></a><br><a href="crates/solana_adapter/"><code>crates/solana_adapter/</code></a></td></tr>
<tr><td width="210" align="center"><picture><source media="(prefers-color-scheme: dark)" srcset="https://github.com/user-attachments/assets/852c5ebe-7e5d-4f03-aa1f-f2a81bab7898"><img src="https://github.com/user-attachments/assets/f11fc02b-713c-4713-86ab-a191861a1fc4" alt="Aurora" width="150"></picture></td><td>Aurora Intents Swap API for cross-chain quotes (dry estimates by default, live deposit mode when enabled), one-time deposit address generation, and client-side status polling until terminal state.</td><td><a href="crates/aurora_adapter/"><code>crates/aurora_adapter/</code></a><br><a href="web/src/components/QuotePanel.tsx"><code>QuotePanel.tsx</code></a><br><a href="web/src/components/LiveDeposit.tsx"><code>LiveDeposit.tsx</code></a></td></tr>
<tr><td width="210" align="center"><picture><source media="(prefers-color-scheme: dark)" srcset="https://github.com/user-attachments/assets/36a1b317-e498-428d-86e5-fd7beadfffad"><img src="https://github.com/user-attachments/assets/90b8aa3d-2243-48a5-9c7a-e550613a1d7e" alt="Privy" width="120"></picture></td><td>Email authentication and invisible embedded Solana wallet provisioning. Users sign vault creation and contribution transactions without managing seed phrases or keypairs.</td><td><a href="web/src/components/wallet-privy.tsx"><code>wallet-privy.tsx</code></a><br><a href="web/src/components/wallet.tsx"><code>wallet.tsx</code></a></td></tr>
</table>

## The problem

Universities take tuition deposits before a visa decision is made. When a visa is refused, getting that deposit back is slow or never happens. In South Korea, about 400 students at Singyeongju University and about 170 students at Woongji College waited months for refunds after visa denials, pushing the education ministry to mandate refund rules. In the UK, tens of thousands of student visa applications are refused or withdrawn each year, and universities hold the deposit either way. A rule in law still depends on the university choosing to pay; Arrhabon makes the custody the rule.

![A student sits waiting outside locked university gates with a denied visa letter, while their tuition deposit sits stuck inside](https://github.com/user-attachments/assets/f5cfbc45-05bc-4f67-8263-931692435684)

| Outlet | Date | What it reports | Link |
|---|---|---|---|
| Korea Herald | 7 Oct 2026 | About 400 foreign students at Singyeongju University owed 2.8bn won ($2.07M) combined; about 170 at Woongji College owed 900M won; 115 complaints over five years | https://www.koreaherald.com/article/10895948 |
| ICEF Monitor | 10 Sep 2026 | Korea education ministry requires refunding tuition paid by students whose visa is denied; junior colleges delayed refunds up to 1bn won ($745k) | https://monitor.icef.com/2026/09/korean-government-introduces-new-quality-controls-for-universities-and-agents/ |
| The PIE News | 5 Oct 2026 | UK, year to June 2026: 19,608 student visas refused and 11,500 applications withdrawn; Pakistan Q1 refusal rate 39.3% | https://thepienews.com/in-focus-how-uk-unis-are-staying-under-the-5-bca-threshold/ |

## What it does

![Five steps of Arrhabon: student opens a vault, family contributes, visa decision arrives, paid to university if issued, refunded to every funder if denied](https://github.com/user-attachments/assets/7013d98a-e352-48fb-957a-5938601719c2)

1. A student opens a vault: names a registered university, a registered attestor, a target amount in USDC and a deadline.
2. The student and family members anywhere contribute. Each funder's money is recorded separately.
3. Visa issued: the attestor confirms and the whole vault goes to the university in one transaction.
4. Visa denied, or the deadline passes with no decision: the vault switches to refunding, and anyone (the crank, the funder, a stranger) can trigger each funder's refund.

<p align="center">
  <img src="https://github.com/user-attachments/assets/a57c0f85-78d3-47ec-bd7f-e68d58ba3554" alt="Visa issued: the stone gateway opens, the empty bell jar lifts, and the tuition coin rolls through to the university" width="49%">
  <img src="https://github.com/user-attachments/assets/bcc7d607-cc3e-47b5-9a22-7be302317f0d" alt="Visa denied: the university gates stay closed and three tuition coins travel along the dashed return path back to the family homes" width="49%">
</p>

### One student, one number

A student admitted to Singyeongju University needs to deposit about 7M won (about $5,200, the average of 2.8bn won across the 400 foreign students reported by Korea Herald). Her family pools the $5,200 in USDC across three contributions into an Arrhabon vault. If the visa is granted, the college receives the full $5,200. If the visa is refused, every dollar returns to the family accounts automatically rather than sitting trapped in an overseas bank account.

## Why the money cannot go anywhere else

- **Custody in a PDA.** USDC sits in a token account owned by the vault program-derived address. No private key can move it; only program instructions can.
- **Two exits only.** Money leaves the vault to the university token account after `confirm_visa`, or to recorded funders after the vault is `Refunding`. No instruction sends funds anywhere else, including to the operator. Sweep moves accidental non-contribute tokens to the rightful recipient (university after release, student after refunds).
- **Release only after attestation.** `confirm_visa` must be signed by the vault attestor, the vault must be open and fully funded, both attestor and university must be active in the registry, and it fails after the deadline.
- **Permissionless expiry and refund.** `expire` and `refund` require no special signer. If the backend disappears, a funder can still trigger them. Each funder has a separate `Contribution` account, so refunds never loop over unbounded lists. My crank skips any funder without a token account yet so it never pays rent; that refund stays open for the funder to claim.
- **Registry.** Universities and attestors must be registered and active in an on-chain registry. A student cannot name themselves as attestor, and student, university and attestor must be three distinct parties.
- **Legacy SPL Token only.** Fixed in program Config as a 6-decimal legacy SPL Token mint; Token-2022 mints are rejected.
- **Deadline cap.** Deadlines are capped at 365 days ahead so funds cannot be parked indefinitely.
- **1 USDC minimum.** Each contribution must be at least 1 USDC to stop dust spam. A contribution can never exceed the target; excess is rejected.
- **Rent returned.** Once refunded or released, anyone can close a `Contribution` account and rent returns to that funder. The student closes the empty vault and token account and receives both rents back.

## Trust model

- **The attestor is the one trust point.** Someone has to verify whether the visa was issued. A dishonest attestor could release a vault for a student whose visa was refused (only if fully funded and before deadline), or deny a visa that was issued (funds return to funders, never to the attestor). In this demo the attestor is a labeled demo key. Production uses the university admissions office together with the student as a multisig, or a licensed agent.
- **`DEMO_ATTESTOR=true` is for the devnet demo only.** It lets anyone call `POST /demo/attest` to confirm or deny vaults that name the demo attestor, so visitors can try both outcomes with test tokens. Vaults naming any other attestor are unaffected. Turn it off for anything that holds real money.
- **Registry admin equals program upgrade authority.** Only the upgrade authority can initialize Config and register or deactivate universities and attestors. Deactivating a party blocks new vaults and releases naming it, but never blocks refunds or expiries.
- **Program upgradeability.** The program is upgradeable for now. Freezing upgrade authority or moving to a multisig is the step before real money.
- **Devnet only.** Runs on Solana devnet with test USDC. Not audited.

## Aurora Intents

A relative holding USDC on other chains (such as Base, Arbitrum or Ethereum) can fund a vault through Aurora Intents without bridging or changing networks manually:

- **Dry quotes by default.** `POST /funding/quote` returns dry price-only estimates (`dry: true`), showing the guaranteed minimum USDC on Solana (`min_usdc_out`), the network fee, and quote validity. No deposit address is created.
- **Live deposit mode.** When the API runs with `AURORA_LIVE=true`, the quote reserves a real one-time deposit address. The web app renders `LiveDeposit.tsx` showing the deposit address in monospace, a copy button, exact transfer instructions, and guaranteed output floor.
- **Live swap status polling.** The client polls `GET /funding/status?deposit_address=...` every 5 seconds until a terminal state is reached: `pending`, `success`, `refunded`, or `failed`. Polling automatically stops on completion, unmount, or quote expiry.
- **Settlement today, vault next.** A live swap settles real USDC to the funder's own wallet on Solana mainnet, shown in the demo (5 USDC sent on Base, 4.76 USDC landed). The demo vault runs on devnet, so the funder contributes in a separate step for now. Crediting the vault automatically (the swap's recipient becoming the vault itself) is the next step, not a working feature yet.
- **Mainnet vs devnet.** Aurora Intents operates on mainnet, whereas this demo vault runs on devnet with test tokens. A live swap delivers real USDC to the funder's Solana address on mainnet; the funder then contributes it to the vault. Paying straight into the vault is the next step (see What's next).
- **Proof: live swap [5 USDC on Base, 4.76 USDC delivered on Solana mainnet (Solscan)](https://solscan.io/account/BcHo2wHfKUq2FUv1obEoNSU9hwnzqChi4XExVNGmh8Jz#transfers)**

## Screenshots

![Arrhabon home page showing the problem, how it works, and active deposit vaults](https://github.com/user-attachments/assets/16c7c382-d5e2-454d-8730-b5d232fc91d9)

![Vault page showing raised tuition funds, target deadline countdown, and family contribution records](https://github.com/user-attachments/assets/2bcab010-fa3f-4b99-97af-7d3211de0264)

![Vault page after automatic refund showing full return of funds to every contributor](https://github.com/user-attachments/assets/bb43b1f6-3da1-4f59-80ec-ca59cf6f242c)

![Public test funds faucet page for judges to claim devnet SOL and test USDC](https://github.com/user-attachments/assets/14083536-41be-4c69-a7bf-e452ec234b2d)

![University proof page showing held deposits and automated release conditions](https://github.com/user-attachments/assets/76b23101-1cc5-4004-a1f4-b9a340ecb578)

## Project layout

Money rules live in one core crate (`domain`) that owns the vault lifecycle, custody, release and refund. Funding (getting money in, including cross-chain through Aurora) never decides where money goes out. Attestation decides who may confirm a visa outcome.

Dependencies point inward. `api` depends on `app`, `app` depends on `domain`, and adapters implement ports defined in `app`. The on-chain program depends on `domain` directly and adapts accounts to it. `domain` has no I/O, no async and no Solana types, compiling both for host and Solana VM.

| Path | Layer | What it holds |
|---|---|---|
| `crates/domain` | Domain | `Vault` aggregate, `Contribution`, `Usdc` (micro-units, checked math), `Deadline`, `VaultStatus` state machine, limits, errors |
| `programs/arrhabon` | Adapter (on chain) | Anchor 1.2.1 program: Config, registry, Vault and Contribution PDAs, vault token account; instructions call domain transitions |
| `crates/app` | Application | Commands (`CreateVault`, `Contribute`, `AttestVisa`, `ExpireDue`, `RunRefunds`, `QuoteCrossChainFunding`), queries (`VaultView`, `FunderView`, `UniversityProof`, `RegistryView`), ports (`VaultLedger`, `AttestorSigner`, `SwapQuoter`, `Clock`), in-memory fakes |
| `crates/solana_adapter` | Adapter | `VaultLedger` over `anchor-client`, chain clock, keypair attestor, relay guard; `devnet` CLI for setup and scripted run |
| `crates/aurora_adapter` | Adapter | `SwapQuoter` over Aurora Swap API (or NEAR 1Click) |
| `crates/api` | Composition root | Axum HTTP routes, refund crank, demo attestor endpoint; no business logic |
| `web/` | Client | Next.js + Privy embedded Solana wallets; renders views and requests wallet signatures |

The API never holds a user private key. For create and contribute it returns an unsigned transaction; the user wallet signs and the API relays through `/tx/submit`.

## Run locally

Prerequisites: Rust (stable), Solana CLI, Anchor 1.2.1 (`avm install 1.2.1`), Node.js with pnpm, and optionally Bun. All keys are project keys under `.keys/` (gitignored, mode 600). For full production setup, Docker and secret handling, see [docs/DEPLOY.md](docs/DEPLOY.md).

1. **Clone the repository and build the program.**
   ```sh
   git clone https://github.com/alienx5499/Arrhabon.git
   cd Arrhabon
   anchor build
   ```
   If Homebrew's `cargo` shadows rustup's, run `PATH=/opt/homebrew/opt/rustup/bin:$PATH anchor build`.

2. **Set up devnet.**
   ```sh
   scripts/devnet_setup.sh
   ```
   Creates project keypairs, airdrops SOL, deploys program, creates 6-decimal test USDC mint, mints test tokens, initializes Config, registers university and attestor. Writes addresses to `deploy/devnet.json`.
   Then `scripts/register_demo_parties.sh` registers the demo universities and attestors and prints the web env lines that name them.

3. **Faucet step.**
   Wallets you log in with on the web app can use the **Get test funds** button in the header or [/faucet](https://arrhabon.vercel.app/faucet) to get devnet SOL and test USDC. If funding the deployer during setup, open https://faucet.solana.com (cluster: devnet).

4. **Run the API.**
   ```sh
   DEMO_ATTESTOR=true WEB_ORIGIN=http://localhost:3000 cargo run -p arrhabon-api
   ```
   Listens on `http://127.0.0.1:8080`; `GET /health` shows status.

5. **Run the web app.**
   ```sh
   cd web && pnpm install
   NEXT_PUBLIC_PRIVY_APP_ID=<app-id> NEXT_PUBLIC_API_URL=http://localhost:8080 pnpm dev
   ```
   Open http://localhost:3000.

   **Or run both with one command** (needs [Bun](https://bun.sh)), from the repo root:
   ```sh
   cd web && pnpm install && cd ..
   bun run dev
   ```
   It starts the API (reading settings from a root `.env` if present) and the web app on port 3000, with each log line prefixed `[api]` or `[web]`. Put `NEXT_PUBLIC_PRIVY_APP_ID` and `NEXT_PUBLIC_API_URL` in `web/.env.local`.

## Tests

```sh
# All Rust crates (run anchor build first: the program tests load target/deploy/arrhabon.so)
cargo test --workspace

# Web app (lint, typecheck, unit tests)
cd web && pnpm lint && pnpm typecheck && pnpm test
```

## Deploy

Detailed deployment instructions are documented in [docs/DEPLOY.md](docs/DEPLOY.md):
- **API:** Render Docker web service with mounted secret keypairs (`crank.json`, `attestor.json`, `faucet.json`).
- **Web:** Vercel deployment pointing at the Render API and Solana devnet RPC.
- **On-chain:** Solana devnet program deployed via `anchor build` and `scripts/devnet_setup.sh`.

## Limits

- **Devnet and test USDC only.** Arrhabon runs on Solana devnet with simulated USDC tokens. It has not been audited for mainnet deployment.
- **Aurora Intents network boundary.** Aurora Intents operates on EVM mainnet (Base, Arbitrum, Ethereum), whereas Arrhabon vaults run on Solana devnet. A live swap delivers real USDC to the user wallet on mainnet, which cannot directly fund a devnet test vault.
- **Attestor is a single signer.** Each vault designates exactly one attestor key as its verification source.
- **Cancel not exposed in client.** While the Anchor program includes a `cancel` instruction for students before contributions begin, it is not wired to an API endpoint or web UI action in this release.
- **365-day deadline cap.** Deadlines cannot exceed 365 days from vault creation (`MAX_VAULT_DURATION_SECS`).
- **1 USDC minimum.** Contributions must be at least 1 USDC (`MIN_CONTRIBUTION = 1_000_000`) to prevent dust spam.
- **Program upgradeability.** The Solana program is still upgradeable.

## What's next

- **Admissions multisig attestor.** Replace the single attestor key with a multisig requiring signatures from both university admissions and the student.
- **Pay straight into the vault from any chain.** Aurora runs on mainnet only, so today a swap delivers USDC to the funder's Solana wallet and the funder contributes in a second step. On mainnet, the swap's recipient becomes the vault itself (Aurora Intents Deposits or Intents Connect calling `contribute`), so a relative on Base pays tuition in one transfer and a failed swap still refunds to their own address.
- **Immutable program deployment.** Freezing the program upgrade authority or transferring ownership to a distributed governance multisig prior to mainnet launch.

## Regulation and tax notes

Arrhabon is non-custodial software. Funds remain in program-derived accounts on the Solana blockchain governed exclusively by open-source Anchor program rules. Contributors retain full cryptographic claim to their deposits until confirmed release or refund conditions are met.

## License

[MIT](LICENSE) &copy; 2026 Prabal Patra
