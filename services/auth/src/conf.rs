use std::{
    net::{IpAddr, Ipv4Addr},
    path::PathBuf,
};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct AuthServerConfig {
    pub bind_address: Ipv4Addr,
    pub port: u32,

    pub login_database: String,
}

impl AuthServerConfig {
    pub fn read(path: PathBuf) -> Result<Self> {
        let file = std::fs::File::open(path)?;
        serde_yaml::from_reader(file).context("could not read yaml file")
    }

    pub fn write(&self, path: PathBuf) -> Result<()> {
        let file = std::fs::File::create(path)?;
        serde_yaml::to_writer(file, self).context("could not write yaml file")
    }
}
