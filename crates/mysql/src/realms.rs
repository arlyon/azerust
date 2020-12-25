use std::time::{Duration, SystemTime};
use async_std::{sync::RwLock, prelude::FutureExt};

use async_trait::async_trait;
use game::realms::{Realm, RealmList};

pub struct MySQLRealmList {
    next_update: RwLock<SystemTime>,
    update_interval: Duration,
    pool: sqlx::MySqlPool,
    realms: RwLock<Vec<Realm>>,
}

impl MySQLRealmList {
     pub async fn new(connect: &str, update_interval: Duration) -> Result<Self, sqlx::Error> {
        Ok(Self {
            pool: sqlx::MySqlPool::connect(connect).await?,
            update_interval,
            next_update: RwLock::new(SystemTime::now()),
            realms: RwLock::new(vec![]),
        })
    }
}

#[async_trait]
impl RealmList for MySQLRealmList {
    async fn realms(&self) -> Vec<Realm> {
        let now = SystemTime::now();
        if now > *self.next_update.read().await {
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
}
