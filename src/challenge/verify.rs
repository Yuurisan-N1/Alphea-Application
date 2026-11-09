use anyhow::{anyhow, Result};
use serde_json::{json, Value};

use crate::net::device::Device;
use crate::net::{client, endpoint};
use crate::support::error::refusal;

#[derive(Clone)]
pub struct Tokens {
    pub access_token: String,
    pub refresh_token: String,
}

pub async fn send(
    shared: &wreq::Client,
    dev: &Device,
    email: &str,
    id: &str,
    code: &str,
) -> Result<Tokens> {
    let head = client::plain(dev)?;
    let body = json!({"email": email, "challengeId": id, "code": code});
    let (status, value) = client::call(shared, &endpoint::verify_challenge(), head, &body).await?;
    if status != 200 {
        return Err(anyhow!("{}", refusal(status)));
    }
    let nested = value.get("session").unwrap_or(&value);
    let access = nested
        .get("bearerToken")
        .or_else(|| nested.get("accessToken"))
        .or_else(|| nested.get("access_token"))
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    let refresh = nested
        .get("refreshToken")
        .or_else(|| nested.get("refresh_token"))
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    if access.is_empty() {
        return Err(anyhow!("the reply carried no session token"));
    }
    Ok(Tokens {
        access_token: access,
        refresh_token: refresh,
    })
}
