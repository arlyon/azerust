use std::{
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use async_std::{prelude::FutureExt, sync::RwLock};
use async_trait::async_trait;
use azerust_game::realms::{Realm, RealmFlags, RealmId, RealmList, RealmListError};
use sqlx::{query, query_as, MySqlPool};
use tracing::{debug, trace};

#[derive(Clone)]
pub struct MySQLRealmList {
    next_update: Arc<RwLock<SystemTime>>,
    update_interval: Duration,
    pool: sqlx::MySqlPool,
    realms: Arc<RwLock<Vec<Realm>>>,
}

impl MySQLRealmList {
    pub fn new(pool: MySqlPool, update_interval: Duration) -> Self {
        debug!("Starting realmlist service");
        Self {
            pool,
            update_interval,
            next_update: Arc::new(RwLock::new(SystemTime::now())),
            realms: Arc::new(RwLock::new(vec![])),
        }
    }
}

#[async_trait]
impl RealmList for MySQLRealmList {
    async fn realms(&self) -> Vec<Realm> {
        let now = SystemTime::now();
        if now > *self.next_update.read().await {
            debug!("Refreshing realm list");
            if let Ok(realms) = query_as!(
                Realm,
                "SELECT id as 'id: _', name, icon as 'realm_type: _', gamebuild as build, address as 'external_address', localAddress as 'local_address: _', localSubnetMask as 'local_subnet_mask: _', port, flag as 'flags: _', timezone, population FROM realmlist WHERE flag <> 3 ORDER BY id"
            )
            .fetch_all(&self.pool)
            .await {
                let (mut self_realms, mut self_next_update) = self.realms.write().join(self.next_update.write()).await;
                *self_realms = realms;
                *self_next_update = now + self.update_interval;
            }
        };

        self.realms.read().await.clone()
    }

    async fn update_status(&self, online: Vec<(u8, RealmFlags)>) -> Result<(), RealmListError> {
        for (id, flag) in online {
            query!(
                "insert into realmlist(id, flag) values(?, ?) on duplicate key update flag = values(`flag`)",
                id, flag as u8
            ).execute(&self.pool).await.map_err(|e| RealmListError::PersistError(e.to_string()))?;
        }

        Ok(())
    }

    async fn set_uptime(
        &self,
        id: RealmId,
        start: SystemTime,
        population: u32,
    ) -> Result<(), RealmListError> {
        let start_u32: u32 = start
            .duration_since(UNIX_EPOCH)
            .expect("no time can be smaller than UNIX_EPOCH")
            .as_secs()
            .try_into()
            .map_err(|_| RealmListError::StartTimeTooLarge)?;

        let uptime = SystemTime::now()
            .duration_since(start)
            .map_err(|_| RealmListError::StartTimeInFuture)?;

        let uptime_u32: u32 = uptime
            .as_secs()
            .try_into()
            .expect("server won't be up for 40 years");

        trace!("setting uptime for realm {:?} to {:?}", id, uptime);
        query!(
            "INSERT INTO uptime
        (
            realmid, starttime, uptime, maxplayers, revision
        )
    VALUES
        (?, ?, ?, ?, 'azerust-0.1.0')
    ON DUPLICATE KEY UPDATE
        uptime = VALUES(uptime), maxplayers = VALUES(maxplayers)
    ",
            id,
            start_u32,
            uptime_u32,
            population
        )
        .execute(&self.pool)
        .await
        .map_err(|e| RealmListError::PersistError(e.to_string()))?;

        Ok(())
    }
}
