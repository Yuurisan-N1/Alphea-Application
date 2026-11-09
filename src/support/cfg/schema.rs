use serde::Deserialize;

fn default_sleep() -> u64 {
    3600
}

#[derive(Clone, Deserialize)]
pub struct Settings {
    #[serde(default = "default_sleep")]
    pub sleep_seconds: u64,
    #[serde(default)]
    pub referral_code: String,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            sleep_seconds: default_sleep(),
            referral_code: String::new(),
        }
    }
}

#[derive(Clone, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub settings: Settings,
}
