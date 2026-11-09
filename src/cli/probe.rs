use anyhow::Result;

use crate::support::log::{describe, lg, lr, ly};
use crate::Ctx;

pub async fn run(ctx: &Ctx) -> Result<()> {
    ly("Probe mode runs read only so nothing is claimed");
    let sessions = {
        let shared = std::sync::Arc::new(Ctx {
            cfg: ctx.cfg.clone(),
            client: ctx.client.clone(),
            store: std::sync::Mutex::new(ctx.store.lock().unwrap().clone()),
            accounts: ctx.accounts.clone(),
            sessions: std::sync::Mutex::new(ctx.sessions.lock().unwrap().clone()),
        });
        crate::run::cycle::ready_sessions(&shared).await
    };
    for account in &ctx.accounts {
        let Some(session) = sessions.get(&account.email) else {
            lr("The account holds no cached session so the probe is skipped");
            continue;
        };
        match crate::activity::earnings::read(
            &ctx.client,
            &crate::net::device::for_email(&account.email),
            &session.access_token,
        )
        .await
        {
            Ok(earnings) => lg(&format!(
                "The account holds {} valid mining seconds and {} points today",
                earnings.valid_seconds,
                crate::support::log::num(earnings.points)
            )),
            Err(err) => lr(&format!(
                "The earnings were not read because {}",
                describe(&err)
            )),
        }
        match crate::reward::summary::read(
            &ctx.client,
            &crate::net::device::for_email(&account.email),
            &session.access_token,
        )
        .await
        {
            Ok(summary) => lg(&format!(
                "The reward summary shows {} points in total",
                crate::support::log::num(summary.total)
            )),
            Err(err) => lr(&format!(
                "The reward summary was not read because {}",
                describe(&err)
            )),
        }
    }
    Ok(())
}
