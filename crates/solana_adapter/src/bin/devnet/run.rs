//! End-to-end cluster run through the app use cases: expiry with refunds, then a release.

use std::time::Duration;

use anchor_client::Signer;
use arrhabon::state::id;
use arrhabon_app::commands::{
    AttestVisa, Contribute, CreateVault, EXPIRY_GRACE_SECS, ExpireDue, RunRefunds,
};
use arrhabon_app::ports::{Clock, VaultLedger, VisaOutcome};
use arrhabon_domain::{AccountId, Deadline, Usdc, VaultStatus};
use arrhabon_solana_adapter::{
    ChainClock, Keypair, KeypairAttestor, Network, SolanaLedger, sign_tx,
};

use crate::{Keys, Res};

const USDC: u64 = 1_000_000;
const TARGET: u64 = 1_000 * USDC;

fn step(net: Network, label: &str, sig: &str) {
    println!("{label:<28} {}", net.explorer_tx(sig));
}

struct Demo {
    net: Network,
    ledger: SolanaLedger,
    clock: ChainClock,
    university: AccountId,
    attestor: AccountId,
}

impl Demo {
    async fn create(
        &self,
        student: &Keypair,
        nonce: u64,
        secs: i64,
        label: &str,
    ) -> Res<AccountId> {
        self.clock.sync(self.ledger.chain()).await?;
        let built = CreateVault {
            student: id(&student.pubkey()),
            university: self.university,
            attestor: self.attestor,
            target: Usdc::new(TARGET),
            deadline: Deadline::new(self.clock.now() + secs),
            nonce,
        }
        .execute(&self.ledger, &self.clock)
        .await?;
        let sig = self.ledger.submit(sign_tx(&built.tx, student)?).await?;
        println!("vault {}", bs58::encode(built.vault.0).into_string());
        step(self.net, label, &sig.0);
        Ok(built.vault)
    }

    async fn contribute(&self, vault: AccountId, funder: &Keypair, usdc: u64, label: &str) -> Res {
        let tx = Contribute {
            vault,
            funder: id(&funder.pubkey()),
            amount: Usdc::new(usdc * USDC),
        }
        .execute(&self.ledger, &self.clock)
        .await?;
        step(
            self.net,
            label,
            &self.ledger.submit(sign_tx(&tx, funder)?).await?.0,
        );
        Ok(())
    }

    async fn status(&self, vault: AccountId) -> Res<VaultStatus> {
        Ok(self
            .ledger
            .vault(vault)
            .await?
            .ok_or("vault vanished")?
            .status())
    }

    /// Polls the Clock sysvar until the crank may expire the vault.
    async fn wait_past(&self, deadline: i64) -> Res {
        loop {
            let now = self.ledger.chain().chain_time().await?;
            if now > deadline + EXPIRY_GRACE_SECS {
                return Ok(());
            }
            println!(
                "waiting for chain time: {}s to go",
                deadline + EXPIRY_GRACE_SECS + 1 - now
            );
            tokio::time::sleep(Duration::from_secs(10)).await;
        }
    }
}

pub async fn run(keys: Keys, net: Network, url: &str) -> Res {
    let ledger = SolanaLedger::connect(net, url, keys.crank).await?;
    let clock = ChainClock::default();
    clock.sync(ledger.chain()).await?;
    let nonce = clock.now() as u64;
    let demo = Demo {
        net,
        ledger,
        clock,
        university: id(&keys.university.pubkey()),
        attestor: id(&keys.attestor.pubkey()),
    };
    let [f1, f2] = &keys.funders;

    println!("-- path B: deadline passes, crank refunds every funder");
    let vault = demo
        .create(&keys.student, nonce, 60, "create_vault (60 s)")
        .await?;
    demo.contribute(vault, f1, 600, "contribute 600 USDC (f1)")
        .await?;
    demo.contribute(vault, f2, 400, "contribute 400 USDC (f2)")
        .await?;
    let deadline = demo
        .ledger
        .vault(vault)
        .await?
        .ok_or("vault missing")?
        .deadline()
        .unix();
    demo.wait_past(deadline).await?;
    demo.clock.sync(demo.ledger.chain()).await?;
    for o in ExpireDue.execute(&demo.ledger, &demo.clock).await? {
        report(net, "expire", o.item == vault, &o.result);
    }
    for o in RunRefunds.execute(&demo.ledger).await? {
        report(net, "refund", o.item.vault == vault, &o.result);
    }
    println!("vault status: {:?}", demo.status(vault).await?);

    println!("-- path A: visa issued, vault released to the university");
    let vault = demo
        .create(&keys.student, nonce + 1, 600, "create_vault (10 min)")
        .await?;
    demo.contribute(vault, f1, 1_000, "contribute 1000 USDC (f1)")
        .await?;
    let attestor = KeypairAttestor(keys.attestor);
    let issued = AttestVisa {
        vault,
        outcome: VisaOutcome::Issued,
    };
    step(
        net,
        "confirm_visa",
        &issued
            .execute(&demo.ledger, &attestor, &demo.clock)
            .await?
            .0,
    );
    println!("vault status: {:?}", demo.status(vault).await?);
    Ok(())
}

fn report<E: std::fmt::Debug>(
    net: Network,
    what: &str,
    ours: bool,
    r: &Result<arrhabon_app::commands::Done, E>,
) {
    use arrhabon_app::commands::Done;
    let tag = if ours {
        what.to_string()
    } else {
        format!("{what} (older vault)")
    };
    match r {
        Ok(Done::Submitted(sig)) => step(net, &tag, &sig.0),
        Ok(Done::AlreadyApplied) => println!("{tag:<28} already applied"),
        Ok(Done::LeftForFunder) => println!("{tag:<28} left for the funder to claim"),
        Err(e) => println!("{tag:<28} failed: {e:?}"),
    }
}
