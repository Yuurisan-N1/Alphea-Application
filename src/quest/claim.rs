use anyhow::{anyhow, Result};
use serde_json::json;

use crate::net::device::Device;
use crate::net::{client, endpoint};
use crate::support::error::refusal;

pub async fn send(shared: &wreq::Client, dev: &Device, token: &str, id: &str) -> Result<bool> {
    let head = client::bearer(dev, token)?;
    let body = json!({"questId": id});
    let (status, _value) = client::call(shared, &endpoint::claim_quest(), head, &body).await?;
    if status == 200 || status == 409 {
        return Ok(status == 200);
    }
    Err(anyhow!("{}", refusal(status)))
}
