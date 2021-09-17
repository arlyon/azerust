use anyhow::{Context, Result};
use async_std::{net::Ipv4Addr, path::PathBuf};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct AuthServerConfig {
    pub bind_address: Ipv4Addr,
    pub port: u16,
    pub api_port: Option<u16>,

    pub login_database: String,
}

impl AuthServerConfig {
    pub async fn read(path: &PathBuf) -> Result<Self> {
        let file = std::fs::File::open(path)?;
        serde_yaml::from_reader(file).context("could not read yaml file")
    }

    pub async fn write(&self, path: &PathBuf) -> Result<()> {
        let file = std::fs::File::create(path)?;
        serde_yaml::to_writer(file, self).context("could not write yaml file")
    }
}
