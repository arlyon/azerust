use std::{net::Ipv4Addr, path::PathBuf};

use anyhow::{Context, Result};
use azerust_game::realms::RealmId;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct WorldServerConfig {
    pub bind_address: Ipv4Addr,
    pub port: u32,

    pub auth_server_address: String,

    pub character_database: String,
    pub auth_database: String,
    pub world_database: String,

    pub realm_id: RealmId,
    pub data_dir: u32,
}

impl WorldServerConfig {
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
