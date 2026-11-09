use anyhow::{anyhow, Result};
use serde_json::json;

use crate::net::device::Device;
use crate::net::{client, endpoint};
use crate::support::error::refusal_with;

pub async fn redeem(shared: &wreq::Client, dev: &Device, token: &str, code: &str) -> Result<()> {
    let head = client::bearer(dev, token)?;
    let body = json!({ "token": code });
    let (status, _value, reason) =
        client::call_reason(shared, &endpoint::redeem_invitation(), head, &body).await?;
    if status == 200 {
        return Ok(());
    }
    if status == 400 && reason.contains("already_exists") {
        return Err(anyhow!("this account already joined a referral"));
    }
    Err(anyhow!("{}", refusal_with(status, &reason)))
}
