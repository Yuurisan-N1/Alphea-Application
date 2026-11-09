use anyhow::{anyhow, Result};
use serde_json::{json, Value};

use crate::net::device::Device;
use crate::net::{client, endpoint};
use crate::support::error::refusal;

pub struct Challenge {
    pub id: String,
}

pub async fn send(shared: &wreq::Client, dev: &Device, email: &str) -> Result<Challenge> {
    let head = client::plain(dev)?;
    let body = json!({"email": email, "delivery": 1});
    let (status, value) = client::call(shared, &endpoint::request_challenge(), head, &body).await?;
    if status != 200 {
        return Err(anyhow!("{}", refusal(status)));
    }
    let id = value
        .get("challengeId")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    if id.is_empty() {
        return Err(anyhow!("the reply carried no challenge id"));
    }
    Ok(Challenge { id })
}
