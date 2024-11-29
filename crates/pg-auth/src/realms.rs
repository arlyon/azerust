use std::{
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use azerust_game::realms::{Realm, RealmFlags, RealmId, RealmList, RealmListError, RealmType};
use tokio::sync::RwLock;
use tracing::{debug, instrument, trace};

use crate::cornucopia::queries::realms;

#[derive(Clone)]
pub struct PgRealmList {
    next_update: Arc<RwLock<SystemTime>>,
    update_interval: Duration,
    pool: deadpool_postgres::Pool,
    realms: Arc<RwLock<Vec<Realm>>>,
}

impl PgRealmList {
    pub fn new(pool: deadpool_postgres::Pool, update_interval: Duration) -> Self {
        debug!("Starting realmlist service");
        Self {
            pool,
            update_interval,
            next_update: Arc::new(RwLock::new(SystemTime::now())),
            realms: Arc::new(RwLock::new(vec![])),
        }
    }
}

impl RealmList for PgRealmList {
    #[instrument(skip(self))]
    async fn realms(&self) -> Vec<Realm> {
        let now = SystemTime::now();
        if now > *self.next_update.read().await {
            debug!("Refreshing realm list");

            if let Ok(realms) = realms::refresh_realmlist()
                .bind(&self.pool.get().await.unwrap())
                .map(|realm| Realm {
                    id: RealmId(realm.id),
                    build: realm.build,
                    external_address: realm.external_address.to_owned(),
                    local_address: realm.local_address.to_owned(),
                    local_subnet_mask: realm.local_subnet_mask.to_owned(),
                    name: realm.name.to_owned(),
                    flags: realm.flags,
                    port: realm.port,
                    population: realm.population as f32,
                    realm_type: match realm.realm_type {
                        0 => RealmType::Normal,
                        1 => RealmType::PVP,
                        6 => RealmType::RP,
                        8 => RealmType::RPPvP,
                        _ => panic!(),
                    },
                    timezone: realm.timezone,
                })
                .all()
                .await
            {
                let (mut self_realms, mut self_next_update) =
                    futures::join!(self.realms.write(), self.next_update.write());
                *self_realms = realms;
                *self_next_update = now + self.update_interval;
            }
        };

        self.realms.read().await.clone()
    }

    #[instrument(skip(self))]
    async fn update_status(&self, online: Vec<(u8, RealmFlags)>) -> Result<(), RealmListError> {
        for (id, flag) in online {
            realms::update_realmlist_status()
                .bind(
                    &self.pool.get().await.unwrap(),
                    &id.into(),
                    &(flag as u8).into(),
                )
                .await
                .unwrap();
        }

        Ok(())
    }

    #[instrument(skip(self))]
    async fn set_uptime(
        &self,
        id: RealmId,
        start: SystemTime,
        population: u32,
    ) -> Result<(), RealmListError> {
        let start_i32: i32 = start
            .duration_since(UNIX_EPOCH)
            .expect("no time can be smaller than UNIX_EPOCH")
            .as_secs()
            .try_into()
            .map_err(|_| RealmListError::StartTimeTooLarge)?;

        let uptime = SystemTime::now()
            .duration_since(start)
            .map_err(|_| RealmListError::StartTimeInFuture)?;

        let uptime_i32: i32 = uptime
            .as_secs()
            .try_into()
            .expect("server won't be up for 40 years");

        trace!("setting uptime for realm {:?} to {:?}", id, uptime);

        realms::set_uptime()
            .bind(
                &self.pool.get().await.unwrap(),
                &id.0,
                &start_i32,
                &uptime_i32,
                &(population as i16),
                &"azerust-0.1.0",
            )
            .await
            .map_err(|e| RealmListError::PersistError(e.to_string()))?;

        Ok(())
    }
}
