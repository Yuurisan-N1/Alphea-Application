use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;

#[derive(Serialize, Deserialize, Clone)]
pub struct Device {
    pub user_agent: String,
    pub device_id: String,
    pub device_model: String,
    pub os_version: String,
    pub app_version: String,
}

const UA: &str = "Mozilla/5.0 (Linux; Android 13; SM-S918B) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Mobile Safari/537.36";
const MODEL: &str = "SM-S918B";
const OS: &str = "13";

const HEX: &[u8] = b"0123456789abcdef";

fn random_android_id() -> String {
    let mut out = String::with_capacity(16);
    let seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.subsec_nanos() as u64 ^ d.as_secs())
        .unwrap_or(0);
    let mut state = seed ^ (std::process::id() as u64).wrapping_mul(0x9E3779B97F4A7C15);
    for _ in 0..16 {
        state = state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let idx = ((state >> 33) as usize) % HEX.len();
        out.push(HEX[idx] as char);
    }
    out
}

fn fresh() -> Device {
    Device {
        user_agent: UA.to_string(),
        device_id: random_android_id(),
        device_model: MODEL.to_string(),
        os_version: OS.to_string(),
        app_version: crate::net::client::app_version(),
    }
}

fn load() -> BTreeMap<String, Device> {
    fs::read_to_string(crate::support::consts::net::DEVICE_PATH)
        .ok()
        .and_then(|raw| serde_json::from_str(&raw).ok())
        .unwrap_or_default()
}

fn save(map: &BTreeMap<String, Device>) {
    if let Ok(raw) = serde_json::to_string_pretty(map) {
        let _ = fs::write(crate::support::consts::net::DEVICE_PATH, raw);
    }
}

pub fn for_email(email: &str) -> Device {
    let mut map = load();
    let key = email.to_lowercase();
    if let Some(hit) = map.get(&key) {
        return hit.clone();
    }
    let dev = fresh();
    map.insert(key, dev.clone());
    save(&map);
    dev
}
