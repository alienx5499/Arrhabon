//! `Clock` on chain time: the local clock corrected by its offset to the Clock sysvar.

use std::sync::atomic::{AtomicI64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use arrhabon_app::ports::{Clock, PortResult};

use crate::chain::Chain;

fn local_now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs() as i64)
}

/// `Clock::now` is synchronous, so the offset is measured by `sync` and reused.
// offset drifts with slot timing; call `sync` before each crank run.
#[derive(Debug, Default)]
pub struct ChainClock {
    offset: AtomicI64,
}

impl ChainClock {
    pub async fn sync(&self, chain: &Chain) -> PortResult<()> {
        let chain_now = chain.chain_time().await?;
        self.offset
            .store(chain_now - local_now(), Ordering::Relaxed);
        Ok(())
    }
}

impl Clock for ChainClock {
    fn now(&self) -> i64 {
        local_now() + self.offset.load(Ordering::Relaxed)
    }
}
