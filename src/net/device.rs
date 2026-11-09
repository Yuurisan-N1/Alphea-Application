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

const MODELS: &[(&str, &str)] = &[
    ("SM-S918B", "14"),
    ("SM-S916B", "14"),
    ("SM-S911B", "13"),
    ("SM-A546B", "14"),
    ("SM-A346B", "14"),
    ("SM-A536B", "13"),
    ("Pixel 7", "14"),
    ("Pixel 8", "14"),
    ("Pixel 6a", "13"),
    ("Pixel 7a", "14"),
    ("M2101K6G", "13"),
    ("22120RN86G", "13"),
    ("CPH2451", "14"),
    ("CPH2381", "13"),
    ("V2210", "13"),
    ("2201116SG", "13"),
    ("Infinix X6819", "13"),
    ("itel S23", "12"),
    ("SM-A155F", "14"),
    ("CPH2583", "14"),
];

const CHROME_BUILD: &str = "131.0.6778.200";

const HEX: &[u8] = b"0123456789abcdef";

fn seed_for(email: &str) -> u64 {
    let mut state = 0xCBF29CE484222325u64;
    for byte in email.to_lowercase().as_bytes() {
        state ^= *byte as u64;
        state = state.wrapping_mul(0x100000001B3);
    }
    state ^= std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.subsec_nanos() as u64)
        .unwrap_or(0);
    state
}

fn pick_model(seed: u64) -> usize {
    let mut state = seed;
    state = state
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    ((state >> 33) as usize) % MODELS.len()
}

fn user_agent(model: &str, os: &str) -> String {
    format!(
        "Mozilla/5.0 (Linux; Android {}; {}) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/{} Mobile Safari/537.36",
        os, model, CHROME_BUILD
    )
}

fn random_android_id(seed: u64) -> String {
    let mut out = String::with_capacity(16);
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

fn fresh(email: &str) -> Device {
    let seed = seed_for(email);
    let (model, os) = MODELS[pick_model(seed)];
    Device {
        user_agent: user_agent(model, os),
        device_id: random_android_id(seed),
        device_model: model.to_string(),
        os_version: os.to_string(),
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
    let dev = fresh(email);
    map.insert(key, dev.clone());
    save(&map);
    dev
}
