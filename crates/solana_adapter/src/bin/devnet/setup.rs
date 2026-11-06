//! Idempotent admin setup: Config with the test USDC mint, demo university and attestor.

use std::sync::Arc;

use anchor_client::Signer;
use anchor_lang::prelude::Pubkey;
use arrhabon::state::{Attestor, Config, University};
use arrhabon_solana_adapter::instructions as ixs;
use arrhabon_solana_adapter::{Chain, Network};

use crate::{Keys, REPO, Res};

pub async fn run(keys: &Keys, net: Network, url: &str) -> Res {
    let admin = &keys.deployer;
    let chain = Chain::new(url, Arc::new(admin.insecure_clone()))?;
    let mint = keys.mint.pubkey();
    let (university, attestor) = (keys.university.pubkey(), keys.attestor.pubkey());

    if chain.fetch::<Config>(ixs::config_pda()).await?.is_none() {
        let sig = chain
            .send_as(&[ixs::init_config(admin.pubkey(), mint)], admin)
            .await?;
        println!("init_config        {}", net.explorer_tx(&sig.0));
    }
    if chain
        .fetch::<University>(ixs::university_pda(&university))
        .await?
        .is_none()
    {
        let ix = ixs::register_university(admin.pubkey(), university);
        println!(
            "register_university {}",
            net.explorer_tx(&chain.send_as(&[ix], admin).await?.0)
        );
    }
    if chain
        .fetch::<Attestor>(ixs::attestor_pda(&attestor))
        .await?
        .is_none()
    {
        let ix = ixs::register_attestor(admin.pubkey(), attestor);
        println!(
            "register_attestor  {}",
            net.explorer_tx(&chain.send_as(&[ix], admin).await?.0)
        );
    }
    let config: Config = chain
        .fetch(ixs::config_pda())
        .await?
        .ok_or("Config missing after init_config")?;
    if config.mint != mint {
        return Err(format!("Config holds mint {}, expected {mint}", config.mint).into());
    }
    write_addresses(keys, net, &config)
}

/// Public addresses only.
fn write_addresses(keys: &Keys, net: Network, config: &Config) -> Res {
    let q = |p: Pubkey| format!("\"{p}\"");
    let json = format!(
        "{{\n  \"cluster\": \"{}\",\n  \"program_id\": {},\n  \"usdc_mint\": {},\n  \
         \"config\": {},\n  \"admin\": {},\n  \"university\": {},\n  \"attestor\": {},\n  \
         \"crank\": {},\n  \"student\": {},\n  \"funders\": [{}, {}]\n}}\n",
        net.name(),
        q(arrhabon::ID),
        q(config.mint),
        q(ixs::config_pda()),
        q(config.admin),
        q(keys.university.pubkey()),
        q(keys.attestor.pubkey()),
        q(keys.crank.pubkey()),
        q(keys.student.pubkey()),
        q(keys.funders[0].pubkey()),
        q(keys.funders[1].pubkey()),
    );
    std::fs::write(format!("{REPO}/deploy/{}.json", net.name()), json)?;
    Ok(())
}
