use std::{fs::read_to_string, path::PathBuf};

use once_cell::sync::Lazy;
use serde::Deserialize;

pub static CONFIG: Lazy<Config> = Lazy::new(|| {
    toml::from_str(
        read_to_string(PathBuf::from("canal-router.toml"))
            .unwrap()
            .as_str(),
    )
    .unwrap()
});

#[derive(Deserialize)]
pub struct Config {
    #[serde(default = "default_dev_name")]
    pub dev_name: String,
    #[serde(default = "default_queues")]
    pub queues: usize,
}

fn default_dev_name() -> String {
    "".to_string()
}

fn default_queues() -> usize {
    1
}
