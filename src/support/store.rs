use std::collections::BTreeMap;
use std::io::Write;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct Session {
    pub access_token: String,
    pub refresh_token: String,
    #[serde(default)]
    pub device_credential: String,
}

pub struct Store {
    path: String,
    map: BTreeMap<String, Session>,
}

impl Store {
    pub fn load(path: &str) -> Result<Store> {
        let raw = std::fs::read_to_string(path).unwrap_or_default();
        let map: BTreeMap<String, Session> = serde_json::from_str(&raw).unwrap_or_default();
        Ok(Store {
            path: path.to_string(),
            map,
        })
    }

    pub fn get(&self, email: &str) -> Option<Session> {
        self.map.get(email).cloned()
    }

    pub fn put(&mut self, email: &str, session: Session) -> Result<()> {
        self.map.insert(email.to_string(), session);
        self.save()
    }

    fn save(&self) -> Result<()> {
        let body = serde_json::to_string_pretty(&self.map)?;
        let tmp = format!("{}.tmp", self.path);
        let mut handle = std::fs::File::create(&tmp)
            .with_context(|| "the session cache could not be written")?;
        handle.write_all(body.as_bytes())?;
        handle.sync_all()?;
        std::fs::rename(&tmp, &self.path)?;
        Ok(())
    }
}

impl Store {
    pub fn clone_map(&self) -> std::collections::BTreeMap<String, Session> {
        self.map.clone()
    }
}

impl Clone for Store {
    fn clone(&self) -> Self {
        Store {
            map: self.map.clone(),
            path: self.path.clone(),
        }
    }
}
