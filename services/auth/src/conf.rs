use std::{net::Ipv4Addr, path::PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct AuthServerConfig {
    pub bind_address: Ipv4Addr,
    pub port: u16,
    pub api_port: Option<u16>,
    pub console_port: Option<u16>,

    pub auth_database: String,
}

impl AuthServerConfig {
    pub async fn read(path: &PathBuf) -> Result<Self> {
        let file = std::fs::File::open(path)
            .with_context(|| format!("could not open config at {:?}", path.as_os_str()))?;
        serde_yaml::from_reader(file).context("could not read yaml file")
    }

    pub async fn write(&self, path: &PathBuf) -> Result<()> {
        let file = std::fs::File::create(path)?;
        serde_yaml::to_writer(file, self).context("could not write yaml file")
    }
}
