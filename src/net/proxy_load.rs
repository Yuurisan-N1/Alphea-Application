#![allow(dead_code)]

use anyhow::Result;

use super::proxy::Proxy;

pub fn load(path: &str) -> Result<Option<Proxy>> {
    let raw = std::fs::read_to_string(path).unwrap_or_default();
    for line in raw.lines() {
        let text = line.trim();
        if text.is_empty() || text.starts_with('#') {
            continue;
        }
        return Ok(Some(Proxy::parse(text)?));
    }
    Ok(None)
}

pub fn load_all(path: &str) -> Result<Vec<Proxy>> {
    let raw = std::fs::read_to_string(path).unwrap_or_default();
    let mut out = Vec::new();
    for line in raw.lines() {
        let text = line.trim();
        if text.is_empty() || text.starts_with('#') {
            continue;
        }
        match Proxy::parse(text) {
            Ok(found) => out.push(found),
            Err(_) => continue,
        }
    }
    Ok(out)
}
