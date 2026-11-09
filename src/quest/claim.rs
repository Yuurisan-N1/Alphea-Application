use anyhow::{anyhow, Result};
use serde_json::{json, Value};

use crate::net::device::Device;
use crate::net::{client, endpoint};
use crate::support::error::refusal;

pub async fn send(
    shared: &wreq::Client,
    dev: &Device,
    token: &str,
    id: &str,
    period: &str,
) -> Result<f64> {
    let head = client::bearer(dev, token)?;
    let body = json!({
        "questId": id,
        "periodKey": period,
        "idempotencyKey": crate::support::id::uuid_v4(),
    });
    let (status, value) = client::call(shared, &endpoint::claim_quest(), head, &body).await?;
    if status == 200 {
        let reward = value
            .get("reward")
            .and_then(|node| node.get("micros"))
            .and_then(Value::as_str)
            .and_then(|raw| raw.parse::<f64>().ok())
            .unwrap_or(0.0)
            / 1_000_000.0;
        return Ok(reward);
    }
    if status == 409 {
        return Ok(0.0);
    }
    Err(anyhow!(refusal(status)))
}
