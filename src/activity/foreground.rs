use anyhow::{anyhow, Result};
use serde_json::{json, Value};

use crate::net::device::Device;
use crate::net::{client, endpoint};
use crate::support::error::refusal;

pub struct Session {
    pub id: String,
}

pub async fn start(shared: &wreq::Client, dev: &Device, token: &str) -> Result<Session> {
    let head = client::bearer(dev, token)?;
    let body = json!({
        "deviceId": dev.device_id.clone(),
        "platform": 1,
        "deviceModel": dev.device_model.clone(),
        "osVersion": dev.os_version.clone(),
        "appVersion": client::app_version(),
    });
    let mut value = serde_json::Value::Null;
    let mut status: u16 = 0;
    for _ in 0..10 {
        let (code, reply) =
            client::call(shared, &endpoint::start_session(), head.clone(), &body).await?;
        status = code;
        value = reply.clone();
        if code == 200 {
            break;
        }
        if code != 503 {
            return Err(anyhow!("{}", refusal(code)));
        }
        tokio::time::sleep(std::time::Duration::from_secs(20)).await;
    }
    if status != 200 {
        return Err(anyhow!("{}", refusal(status)));
    }
    let id = value
        .get("sessionId")
        .or_else(|| value.get("session_id"))
        .or_else(|| value.get("session").and_then(|s| s.get("sessionId")))
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    if id.is_empty() {
        return Err(anyhow!("the reply carried no session id"));
    }
    Ok(Session { id })
}

pub async fn end(shared: &wreq::Client, dev: &Device, token: &str, id: &str) -> Result<()> {
    let head = client::bearer(dev, token)?;
    let body = json!({"sessionId": id});
    let (status, _value) = client::call(shared, &endpoint::end_session(), head, &body).await?;
    if status != 200 {
        return Err(anyhow!("{}", refusal(status)));
    }
    Ok(())
}
