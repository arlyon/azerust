use std::{
    sync::Arc,
    time::{Duration, SystemTime},
};

use async_std::{prelude::FutureExt, sync::RwLock};
use async_trait::async_trait;
use azerust_game::realms::{Realm, RealmFlags, RealmList};
use sqlx::MySqlPool;
use tracing::debug;

#[derive(Clone)]
pub struct MySQLRealmList {
    next_update: Arc<RwLock<SystemTime>>,
    update_interval: Duration,
    pool: sqlx::MySqlPool,
    realms: Arc<RwLock<Vec<Realm>>>,
}

impl MySQLRealmList {
    pub async fn new(pool: MySqlPool, update_interval: Duration) -> Result<Self, sqlx::Error> {
        debug!("Starting realmlist service");
        Ok(Self {
            pool,
            update_interval,
            next_update: Arc::new(RwLock::new(SystemTime::now())),
            realms: Arc::new(RwLock::new(vec![])),
        })
    }
}

#[async_trait]
impl RealmList for MySQLRealmList {
    async fn realms(&self) -> Vec<Realm> {
        let now = SystemTime::now();
        if now > *self.next_update.read().await {
            debug!("Refreshing realm list");
            if let Ok(realms) = sqlx::query_as!(
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

    async fn update_status(&self, online: Vec<(u8, RealmFlags)>) -> () {
        for (id, flag) in online {
            sqlx::query!(
                "insert into realmlist(id, flag) values(?, ?) on duplicate key update flag = values(`flag`)",
                id, flag as u8
            ).execute(&self.pool).await;
        }
    }
}
