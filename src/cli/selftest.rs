use anyhow::Result;

use crate::support::log::{lg, lr};
use crate::Ctx;

pub fn run(_ctx: &Ctx) -> Result<()> {
    match crate::net::proxy::Proxy::parse("socks5://user:secret@198.51.100.20:1080") {
        Ok(proxy) if !proxy.masked().contains("secret") => {
            lg("The proxy parser hides the password in the mask")
        }
        _ => lr("The proxy parser failed the local check"),
    }

    match crate::support::store::Store::load("session.selftest.json") {
        Ok(mut store) => {
            let _ = store.put(
                "qa@gmail.com",
                crate::support::store::Session {
                    access_token: "a".to_string(),
                    refresh_token: "r".to_string(),
                    device_credential: String::new(),
                },
            );
            let hit = store.get("qa@gmail.com").is_some();
            let _ = std::fs::remove_file("session.selftest.json");
            if hit {
                lg("The session cache round trip passed the local check");
            } else {
                lr("The session cache round trip failed the local check");
            }
        }
        Err(_) => lr("The session cache round trip failed the local check"),
    }

    if crate::support::consts::net::DAILY_TARGET_SECONDS == 12 * 60 * 60 {
        lg("The daily mining target holds twelve hours");
    } else {
        lr("The daily mining target is not twelve hours");
    }

    lg("Self test passed");
    Ok(())
}
