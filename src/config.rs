use crate::protocol::Device;
use anyhow::Context;
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct Config {
    /// `en` ou `fr` ; par défaut la langue de l'environnement (`LANG`).
    #[serde(default)]
    pub language: Option<String>,
    pub device: DeviceCfg,
    #[serde(default)]
    pub sync: Sync,
}

#[derive(Debug, Deserialize)]
pub struct DeviceCfg {
    pub id: Device,
    pub midi_port_in: usize,
    pub midi_port_out: usize,
}

#[derive(Debug, Deserialize)]
pub struct Sync {
    pub interval: u64,
}

impl Default for Sync {
    fn default() -> Self {
        Sync { interval: 10 }
    }
}

impl Config {
    pub fn load(path: &Path) -> anyhow::Result<Self> {
        let s = std::fs::read_to_string(path).with_context(|| crate::i18n::tf("err.read_config", path.display()))?;
        serde_yaml::from_str(&s).with_context(|| crate::i18n::tf("err.parse_config", path.display()))
    }
}
