use std::collections::HashMap;
use std::io::Write;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;

use crate::support::log::{countdown, describe, gap, lg, lr, ly, mail};
use crate::support::store::Session;
use crate::Ctx;

const HEARTBEAT_SECONDS: u64 = 60;
const TARGET_SECONDS: u64 = crate::support::consts::net::DAILY_TARGET_SECONDS;
const RETRY_SECONDS: u64 = 300;
const SERVICE_WAIT: Duration = Duration::from_secs(15);
const START_RETRY_SECONDS: u64 = 300;

pub async fn run(ctx: &Arc<Ctx>) -> Result<()> {
    let mut cycle: u64 = 0;
    loop {
        cycle += 1;
        ly(&format!("Starting automation cycle number {}", cycle));

        let sessions = ready_sessions(ctx).await;
        if sessions.is_empty() {
            lr("No account could be signed in so the cycle is skipped");
        }

        let mut handles = Vec::new();
        let mut index: usize = 0;
        for account in &ctx.accounts {
            let Some(session) = sessions.get(&account.email) else {
                continue;
            };
            if index > 0 {
                gap();
            }
            index += 1;
            let client = ctx.client.clone();
            let dev = Arc::new(crate::net::device::for_email(&account.email));
            let token = session.access_token.clone();
            let renewal = session.refresh_token.clone();
            let credential = session.device_credential.clone();
            let mail = account.email.clone();
            let shift_mail = mail.clone();
            let spawn_ctx = Arc::clone(ctx);
            handles.push((
                shift_mail,
                tokio::spawn(async move {
                    mine(
                        &client,
                        &dev,
                        &token,
                        &renewal,
                        &credential,
                        &mail,
                        &spawn_ctx,
                        index,
                    )
                    .await
                }),
            ));
        }

        let mut complete = true;
        for (mail, handle) in handles {
            match handle.await {
                Ok(done) => complete &= done,
                Err(_) => {
                    complete = false;
                    lr(&format!(
                        "The mining shift for {} stopped unexpectedly",
                        crate::support::log::mail(&mail)
                    ));
                }
            }
        }

        report_all(ctx, &sessions).await;
        if complete {
            countdown(ctx.cfg.settings.sleep_seconds, "Next cycle starts in");
        } else {
            countdown(RETRY_SECONDS, "Next cycle starts in");
        }
        crate::support::banner::print_banner();
    }
}

async fn with_service_retry<T, F, Fut>(mut step: F) -> Option<T>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T>>,
{
    for attempt in 0..3 {
        match step().await {
            Ok(value) => return Some(value),
            Err(err) => {
                let cause = describe(&err);
                if !cause.contains("service is not answering")
                    && !cause.contains("limiting requests")
                {
                    return None;
                }
                if attempt == 2 {
                    return None;
                }
                tokio::time::sleep(SERVICE_WAIT).await;
            }
        }
    }
    None
}

pub async fn ready_sessions(ctx: &Arc<Ctx>) -> HashMap<String, Session> {
    let cached = ctx.store.lock().unwrap().clone_map();
    let mut out = HashMap::new();
    for account in &ctx.accounts {
        let dev = crate::net::device::for_email(&account.email);
        let Some(session) = cached.get(&account.email) else {
            sign_in_and_store(ctx, &account.email, &dev, &mut out).await;
            continue;
        };

        let mut revived: Option<Session> = None;

        if !session.refresh_token.is_empty() {
            if let Some(fresh) = with_service_retry(|| {
                crate::challenge::refresh::send(&ctx.client, &dev, &session.refresh_token)
            })
            .await
            {
                revived = Some(Session {
                    access_token: fresh.access_token.clone(),
                    refresh_token: fresh.refresh_token.clone(),
                    device_credential: session.device_credential.clone(),
                });
            }
        }

        if revived.is_none() && !session.device_credential.is_empty() {
            if let Some(value) = with_service_retry(|| {
                crate::challenge::device::login(&ctx.client, &dev, &session.device_credential)
            })
            .await
            {
                revived = Some(Session {
                    access_token: value.tokens.access_token.clone(),
                    refresh_token: value.tokens.refresh_token.clone(),
                    device_credential: value.credential.clone(),
                });
            } else if !session.device_credential.is_empty() {
                lr("The device sign in was refused so a new code is needed");
            }
        }

        if revived.is_none()
            && session.refresh_token.is_empty()
            && session.device_credential.is_empty()
        {
            if crate::activity::earnings::read(&ctx.client, &dev, &session.access_token)
                .await
                .is_ok()
            {
                revived = Some(session.clone());
                lg("The session for the account still holds a live access token");
            } else {
                lr("The cached session was refused so a new code is needed");
            }
        }

        match revived {
            Some(saved) => {
                let _ = ctx.store.lock().unwrap().put(&account.email, saved.clone());
                lg("The session for the account was renewed from the cache");
                out.insert(account.email.clone(), saved.clone());
                let dev = crate::net::device::for_email(&account.email);
                report_balance(&ctx.client, &dev, &saved.access_token).await;
            }
            None => {
                sign_in_and_store(ctx, &account.email, &dev, &mut out).await;
            }
        }
    }
    out
}

async fn sign_in_and_store(
    ctx: &Arc<Ctx>,
    email: &str,
    dev: &crate::net::device::Device,
    out: &mut HashMap<String, Session>,
) {
    match sign_in(ctx, email, dev).await {
        Ok(session) => {
            if !ctx.cfg.settings.referral_code.is_empty() {
                match crate::referral::redeem::redeem(
                    &ctx.client,
                    dev,
                    &session.access_token,
                    &ctx.cfg.settings.referral_code,
                )
                .await
                {
                    Ok(()) => lg("The referral code was bound for the account"),
                    Err(err) => lr(&format!(
                        "The referral code was not bound because {}",
                        describe(&err)
                    )),
                }
            }
            let _ = ctx.store.lock().unwrap().put(email, session.clone());
            out.insert(email.to_string(), session.clone());
            report_balance(&ctx.client, dev, &session.access_token).await;
        }
        Err(err) => {
            lr(&format!(
                "The sign in was refused because {}",
                describe(&err)
            ));
        }
    }
}

async fn sign_in(ctx: &Arc<Ctx>, email: &str, dev: &crate::net::device::Device) -> Result<Session> {
    let label = mail(email);
    let challenge = crate::challenge::request::send(&ctx.client, dev, email).await?;
    ly("The verification code was requested");
    crate::support::log::status(&format!("Verification code for {}:", label));
    let code = read_line()?;
    crate::support::log::clear_status();
    let tokens =
        crate::challenge::verify::send(&ctx.client, dev, email, &challenge.id, &code).await?;
    lg("The sign in holds a fresh session");
    let device_credential =
        match crate::challenge::device::register(&ctx.client, dev, &tokens.access_token).await {
            Ok(credential) => credential,
            Err(_) => {
                lr("The device registration was refused so renewal may fail");
                String::new()
            }
        };
    Ok(Session {
        access_token: tokens.access_token,
        refresh_token: tokens.refresh_token,
        device_credential,
    })
}

fn read_line() -> Result<String> {
    let mut line = String::new();
    std::io::stdout().flush()?;
    std::io::stdin().read_line(&mut line)?;
    let code = line.trim().to_string();
    if code.is_empty() {
        return Err(anyhow::anyhow!("the verification code was left empty"));
    }
    Ok(code)
}

async fn beat_once(
    client: &wreq::Client,
    dev: &crate::net::device::Device,
    token: &str,
    id: &str,
) -> Result<crate::activity::heartbeat::Beat, StalledBeat> {
    match tokio::time::timeout(
        Duration::from_secs(HEARTBEAT_SECONDS * 2),
        crate::activity::heartbeat::send(client, dev, token, id),
    )
    .await
    {
        Ok(result) => match result {
            Ok(beat) => Ok(beat),
            Err(fail) => Err(StalledBeat::Failed(fail)),
        },
        Err(_) => Err(StalledBeat::Hung),
    }
}

enum StalledBeat {
    Hung,
    Failed(crate::activity::heartbeat::BeatFail),
}

async fn restart_session(
    client: &wreq::Client,
    dev: &crate::net::device::Device,
    token: &str,
    old: &str,
) -> Option<crate::activity::foreground::Session> {
    let _ = crate::activity::foreground::end(client, dev, token, old).await;
    crate::activity::foreground::start(client, dev, token)
        .await
        .ok()
}

async fn mine(
    client: &wreq::Client,
    dev: &crate::net::device::Device,
    token: &str,
    renewal: &str,
    credential: &str,
    email: &str,
    ctx: &Arc<Ctx>,
    label: usize,
) -> bool {
    let mut valid: u64 = 0;
    let mut beats: u64 = 0;
    let mut session = loop {
        match crate::activity::foreground::start(client, dev, token).await {
            Ok(session) => break session,
            Err(err) => {
                let cause = describe(&err);
                if cause.contains("service is not answering")
                    || cause.contains("limiting requests")
                {
                    ly(&format!(
                        "The mining session waits because {} and the start retries",
                        cause
                    ));
                    tokio::time::sleep(Duration::from_secs(START_RETRY_SECONDS)).await;
                    continue;
                }
                lr(&format!(
                    "The mining session could not start because {}",
                    cause
                ));
                return false;
            }
        }
    };
    lg("The mining session for the account is running");

    claim_quests(client, dev, token).await;
    let mut live = token.to_string();
    let mut live_refresh = renewal.to_string();
    let mut live_credential = credential.to_string();
    let mut rescues: u32 = 0;
    let mut streak: u32 = 0;
    crate::support::log::status("The account waits for the first server beat");
    while valid < TARGET_SECONDS {
        tokio::time::sleep(Duration::from_secs(HEARTBEAT_SECONDS)).await;

        let outcome = match beat_once(client, dev, &live, &session.id).await {
            Ok(beat) => Ok(beat),
            Err(StalledBeat::Hung) => {
                lr("The heartbeat for the account hung so the beat retries");
                continue;
            }
            Err(StalledBeat::Failed(fail)) => Err(fail),
        };
        match outcome {
            Ok(beat) => {
                beats += 1;
                streak = 0;
                if beat.parsed && beat.valid_seconds > 0 {
                    rescues = 0;
                }
                if beat.parsed {
                    valid = beat.valid_seconds;
                } else if beats % 10 == 0 {
                    ly("The heartbeat reply carries no valid seconds so the shift keeps waiting");
                }
                crate::support::log::progress(&format!(
                    "Account {} The mining shift for the account holds {} of {} valid seconds {}",
                    label, valid, TARGET_SECONDS, beats
                ));
            }
            Err(crate::activity::heartbeat::BeatFail::SlotLost) => {
                ly("The mining slot moved to another session so the shift retakes it");
                if let Some(fresh) = restart_session(client, dev, &live, &session.id).await {
                    session = fresh;
                }
                streak = 0;
                continue;
            }
            Err(crate::activity::heartbeat::BeatFail::Refused) => {
                streak += 1;
                if streak < 3 {
                    lg("The heartbeat was refused so the beat is skipped");
                    continue;
                }
                if rescues >= 2 {
                    lr("The mining session was refused so the shift ends early");
                    break;
                }
                rescues += 1;
                let mut revived = false;

                if !live_refresh.is_empty() {
                    if let Some(fresh) = with_service_retry(|| {
                        crate::challenge::refresh::send(client, dev, &live_refresh)
                    })
                    .await
                    {
                        live = fresh.access_token.clone();
                        live_refresh = fresh.refresh_token.clone();
                        revived = true;
                    }
                }

                if !revived && !live_credential.is_empty() {
                    if let Some(value) = with_service_retry(|| {
                        crate::challenge::device::login(client, dev, &live_credential)
                    })
                    .await
                    {
                        live = value.tokens.access_token.clone();
                        live_refresh = value.tokens.refresh_token.clone();
                        live_credential = value.credential.clone();
                        revived = true;
                    } else {
                        live_credential.clear();
                    }
                }

                if revived {
                    let saved = Session {
                        access_token: live.clone(),
                        refresh_token: live_refresh.clone(),
                        device_credential: live_credential.clone(),
                    };
                    let _ = ctx.store.lock().unwrap().put(email, saved);
                    if let Some(fresh) = restart_session(client, dev, &live, &session.id).await
                    {
                        session = fresh;
                    }
                    lg("The session for the account was renewed inside the shift");
                    streak = 0;
                    continue;
                }
                continue;
            }
        }
        if valid >= TARGET_SECONDS {
            break;
        }
    }

    let finished = valid >= TARGET_SECONDS;

    crate::support::log::clear_status();
    match crate::activity::foreground::end(client, dev, &live, &session.id).await {
        Ok(()) => lg("The mining session for the account was closed"),
        Err(err) => lr(&format!(
            "The mining session for the account could not close because {}",
            describe(&err)
        )),
    }

    claim_quests(client, dev, &live).await;
    finished
}

async fn claim_quests(client: &wreq::Client, dev: &crate::net::device::Device, token: &str) {
    let quests = match crate::quest::list::read(client, dev, token).await {
        Ok(quests) => quests,
        Err(err) => {
            lr(&format!(
                "The quest list was not read because {}",
                describe(&err)
            ));
            return;
        }
    };
    for quest in quests {
        if quest.state != "QUEST_STATE_CLAIMABLE" {
            continue;
        }
        match crate::quest::claim::send(client, dev, token, &quest.id, &quest.period).await {
            Ok(reward) => lg(&format!(
                "The quest {} was claimed and it paid {} points",
                crate::support::log::clean_text(&quest.id, "quest"),
                crate::support::log::num(reward)
            )),
            _ => lg("The quest claim was refused"),
        }
    }
}

async fn report_balance(client: &wreq::Client, dev: &crate::net::device::Device, token: &str) {
    match crate::reward::summary::read(client, dev, token).await {
        Ok(summary) => lg(&format!(
            "The point balance stands at {} points",
            crate::support::log::num(summary.total)
        )),
        Err(_) => lr("The point balance was not read"),
    }
}

async fn report_all(ctx: &Arc<Ctx>, sessions: &HashMap<String, Session>) {
    for account in &ctx.accounts {
        let Some(session) = sessions.get(&account.email) else {
            continue;
        };
        let dev = crate::net::device::for_email(&account.email);
        match crate::activity::earnings::read(&ctx.client, &dev, &session.access_token).await {
            Ok(earnings) => lg(&format!(
                "The account holds {} valid mining seconds and {} points",
                earnings.valid_seconds,
                crate::support::log::num(earnings.points)
            )),
            Err(err) => lr(&format!(
                "The earnings were not read because {}",
                describe(&err)
            )),
        }
        if let Ok(summary) =
            crate::reward::summary::read(&ctx.client, &dev, &session.access_token).await
        {
            lg(&format!(
                "The reward summary shows {} points in total",
                crate::support::log::num(summary.total)
            ));
        }
    }
}
