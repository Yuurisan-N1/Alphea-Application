use anyhow::{anyhow, Result};
use serde_json::{json, Value};

use crate::net::device::Device;
use crate::net::{client, endpoint};
use crate::support::error::refusal;

pub struct Earnings {
    pub valid_seconds: u64,
    pub points: f64,
}

pub async fn read(shared: &wreq::Client, dev: &Device, token: &str) -> Result<Earnings> {
    let head = client::bearer(dev, token)?;
    let body = json!({});
    let (status, value) = client::call(shared, &endpoint::earnings(), head, &body).await?;
    if status != 200 {
        return Err(anyhow!("{}", refusal(status)));
    }
    let balance = micros(&value, "balance");
    let today = micros(&value, "earnedToday");
    Ok(Earnings {
        valid_seconds: today as u64,
        points: balance,
    })
}

fn micros(value: &Value, key: &str) -> f64 {
    let Some(node) = value.get(key) else {
        return 0.0;
    };
    let raw = node
        .get("micros")
        .and_then(Value::as_str)
        .unwrap_or_default();
    raw.parse::<f64>().unwrap_or(0.0) / 1_000_000.0
}
