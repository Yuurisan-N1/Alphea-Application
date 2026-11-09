use anyhow::{anyhow, Result};
use serde_json::{json, Value};

use crate::net::device::Device;
use crate::net::{client, endpoint};
use crate::support::error::refusal;

pub struct Summary {
    pub total: f64,
}

pub async fn read(shared: &wreq::Client, dev: &Device, token: &str) -> Result<Summary> {
    let head = client::bearer(dev, token)?;
    let body = json!({});
    let (status, value) = client::call(shared, &endpoint::point_summary(), head, &body).await?;
    if status != 200 {
        return Err(anyhow!("{}", refusal(status)));
    }
    let total = value
        .get("balance")
        .and_then(|node| node.get("micros"))
        .and_then(Value::as_str)
        .and_then(|raw| raw.parse::<f64>().ok())
        .unwrap_or(0.0)
        / 1_000_000.0;
    Ok(Summary { total })
}
