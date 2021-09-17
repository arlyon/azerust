use std::time::Duration;

use anyhow::{Context, Result};
use azerust_game::{accounts::AccountService, realms::RealmList};
use async_std::{net::UdpSocket, prelude::*, stream};
use bincode::Options;
use tracing::{instrument, trace, warn};

use crate::wow_bincode::wow_bincode;

pub struct WorldServer<A: AccountService, R: RealmList> {
    pub accounts: A,
    pub realms: R,
    pub auth_server_address: String,
    pub realm_id: u8
}

impl<A: AccountService, R: RealmList> WorldServer<A, R> {
    pub async fn start(&self) -> Result<()> {
        self
            .auth_server_heartbeat()
            .try_join(self.accept_clients())
            .try_join(self.update())
            .await
            .map(|_| ())
    }

    #[instrument(skip(self))]
    pub async fn auth_server_heartbeat(&self) -> Result<()> {
        let socket = UdpSocket::bind("127.0.0.1:0").await?;
        socket.connect(&self.auth_server_address).await?;

        let population = 0u32;
        
        let mut interval = stream::interval(Duration::from_secs(5));
        while let Some(_) = interval.next().await {
            trace!("sending population heartbeat {}", population);
            let mut buffer = [0u8; 6];
            wow_bincode().serialize_into(&mut buffer[..], &(0u8, self.realm_id, population))?;    
            if let Err(e) = socket.send(&buffer).await {
                warn!("could not send heartbeat to {}", self.auth_server_address);
            }
        }
        
        Ok(())
    }
    
    #[instrument(skip(self))]
    pub async fn accept_clients(&self) -> Result<()> {
        Ok(())
    }
    
    #[instrument(skip(self))]
    pub async fn update(&self) -> Result<()> {
        Ok(())
    }
}
