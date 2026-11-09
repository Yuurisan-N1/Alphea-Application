use serde::Deserialize;

fn default_sleep() -> u64 {
    3600
}

#[derive(Clone, Deserialize)]
pub struct Settings {
    #[serde(default = "default_sleep")]
    pub sleep_seconds: u64,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            sleep_seconds: default_sleep(),
        }
    }
}

#[derive(Clone, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub settings: Settings,
}
