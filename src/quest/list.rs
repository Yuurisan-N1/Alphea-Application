use anyhow::{anyhow, Result};
use serde_json::{json, Value};

use crate::net::device::Device;
use crate::net::{client, endpoint};
use crate::support::error::refusal;

pub struct Quest {
    pub id: String,
    pub state: String,
}

pub async fn read(shared: &wreq::Client, dev: &Device, token: &str) -> Result<Vec<Quest>> {
    let head = client::bearer(dev, token)?;
    let body = json!({});
    let (status, value) = client::call(shared, &endpoint::quests(), head, &body).await?;
    if status != 200 {
        return Err(anyhow!("{}", refusal(status)));
    }
    let mut out = Vec::new();
    let items = value
        .get("quests")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    for item in items {
        let id = item
            .get("questId")
            .or_else(|| item.get("id"))
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        let state = item
            .get("state")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        if id.is_empty() {
            continue;
        }
        out.push(Quest { id, state });
    }
    Ok(out)
}
