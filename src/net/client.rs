use anyhow::Result;
use serde_json::Value;
use wreq::header::{HeaderMap, HeaderValue, CONTENT_TYPE, ORIGIN, USER_AGENT};
use wreq::Client;

use crate::net::device::Device;
use crate::net::proxy::Proxy;
use crate::support::consts::app;

pub fn build(proxy: Option<&Proxy>) -> Result<Client> {
    let mut builder = Client::builder()
        .emulation(wreq_util::Emulation::Chrome131)
        .timeout(std::time::Duration::from_secs(45))
        .gzip(true);
    if let Some(found) = proxy {
        builder = builder.proxy(wreq::Proxy::all(found.url())?);
    }
    Ok(builder.build()?)
}

pub fn plain(dev: &Device) -> Result<HeaderMap> {
    let mut head = HeaderMap::new();
    head.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    head.insert(USER_AGENT, HeaderValue::from_str(&dev.user_agent)?);
    head.insert(ORIGIN, HeaderValue::from_static(app::ORIGIN));
    Ok(head)
}

pub fn bearer(dev: &Device, token: &str) -> Result<HeaderMap> {
    let mut head = plain(dev)?;
    head.insert(
        "authorization",
        HeaderValue::from_str(&format!("Bearer {token}"))?,
    );
    Ok(head)
}

pub fn app_version() -> String {
    "1.0.0".to_string()
}

pub async fn call(
    client: &Client,
    url: &str,
    head: HeaderMap,
    body: &Value,
) -> Result<(u16, Value)> {
    let reply = client.post(url).headers(head).json(body).send().await?;
    let status = reply.status().as_u16();
    let text = reply.text().await?;
    let value = serde_json::from_str(&text).unwrap_or(Value::String(text));
    Ok((status, value))
}

pub async fn call_reason(
    client: &Client,
    url: &str,
    head: HeaderMap,
    body: &Value,
) -> Result<(u16, Value, String)> {
    let reply = client.post(url).headers(head).json(body).send().await?;
    let status = reply.status().as_u16();
    let text = reply.text().await?;
    let value = serde_json::from_str(&text).unwrap_or(Value::String(text.clone()));
    let message = serde_json::from_str::<Value>(&text)
        .ok()
        .and_then(|node| {
            node.get("message")
                .and_then(Value::as_str)
                .map(str::to_string)
        })
        .unwrap_or_default();
    Ok((status, value, message))
}
