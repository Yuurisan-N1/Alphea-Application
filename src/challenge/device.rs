use anyhow::{anyhow, Result};
use serde_json::{json, Value};

use crate::net::device::Device;
use crate::net::{client, endpoint};
use crate::support::error::refusal;

use super::verify::Tokens;

pub async fn register(shared: &wreq::Client, dev: &Device, token: &str) -> Result<String> {
    let head = client::bearer(dev, token)?;
    let body = json!({
        "deviceId": dev.device_id.clone(),
        "platform": 1,
        "deviceModel": dev.device_model.clone(),
        "osVersion": dev.os_version.clone(),
        "appVersion": dev.app_version.clone(),
    });
    let (status, value) = client::call(shared, &endpoint::register_device(), head, &body).await?;
    if status != 200 {
        return Err(anyhow!("{}", refusal(status)));
    }
    let credential = value
        .get("deviceCredential")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    if credential.is_empty() {
        return Err(anyhow!("the reply carried no device credential"));
    }
    Ok(credential)
}

pub struct LoginOut {
    pub tokens: Tokens,
    pub credential: String,
}

pub async fn login(shared: &wreq::Client, dev: &Device, credential: &str) -> Result<LoginOut> {
    let head = client::plain(dev)?;
    let body = json!({
        "deviceCredential": credential,
        "deviceId": dev.device_id.clone(),
    });
    let (status, value) = client::call(shared, &endpoint::device_login(), head, &body).await?;
    if status != 200 {
        return Err(anyhow!("{}", refusal(status)));
    }
    let session = value.get("session").unwrap_or(&value);
    let access = session
        .get("accessToken")
        .or_else(|| session.get("bearerToken"))
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    let refresh = session
        .get("refreshToken")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    if access.is_empty() {
        return Err(anyhow!("the reply carried no session token"));
    }
    let rotated = value
        .get("rotatedDeviceCredential")
        .and_then(Value::as_str)
        .unwrap_or(credential)
        .to_string();
    Ok(LoginOut {
        tokens: Tokens {
            access_token: access,
            refresh_token: refresh,
        },
        credential: rotated,
    })
}
