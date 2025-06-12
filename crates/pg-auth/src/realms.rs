use std::{
    net::IpAddr,
    sync::Arc,
    time::{Duration, SystemTime},
};

use azerust_game::realms::{Realm, RealmFlags, RealmId, RealmList, RealmListError, RealmType};
use time::{OffsetDateTime, PrimitiveDateTime, UtcOffset};
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
    async fn realms(&self) -> Result<Vec<Realm>, RealmListError> {
        let now = SystemTime::now();
        if now > *self.next_update.read().await {
            debug!("Refreshing realm list");

            if let Ok(realms) = realms::refresh_realmlist()
                .bind(&self.pool.get().await.map_err(|e| RealmListError::PersistError(e.to_string()))?)
                .map(|realm| Realm {
                    id: RealmId(realm.id as u32),
                    build: realm.build,
                    external_address: realm.external_address.to_owned(),
                    local_address: realm.local_address.to_owned(),
                    local_subnet_mask: realm.local_subnet_mask.to_owned(),
                    name: realm.name.to_owned(),
                    flags: realm.flags,
                    port: realm.port,
                    population: realm.population,
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
                debug!(realms = realms.len(), "Refreshed realm list");
                let (mut self_realms, mut self_next_update) = futures::join!(self.realms.write(), self.next_update.write());
                *self_realms = realms;
                *self_next_update = now + self.update_interval;
            }
        };

        Ok(self.realms.read().await.clone())
    }

    #[instrument(skip(self))]
    async fn update_status(&self, online: Vec<(u8, RealmFlags)>) -> Result<(), RealmListError> {
        for (id, flag) in online {
            realms::update_realmlist_status()
                .bind(
                    &self.pool.get().await.map_err(|e| RealmListError::PersistError(e.to_string()))?,
                    &(flag as u8),
                    &id.into(),
                )
                .await
                .map_err(|e| RealmListError::PersistError(e.to_string()))?;
        }

        Ok(())
    }

    #[instrument(skip(self))]
    async fn set_uptime(&self, id: RealmId, start: OffsetDateTime, population: u16) -> Result<(), RealmListError> {
        let utc = start.to_offset(UtcOffset::UTC);
        let uptime = OffsetDateTime::now_utc() - start;

        let uptime_u32: u32 = uptime.whole_seconds().try_into().expect("server won't be up for 40 years");

        trace!("setting uptime for realm {:?} to {:?}", id, uptime);

        realms::set_uptime()
            .bind(
                &self.pool.get().await.map_err(|e| RealmListError::PersistError(e.to_string()))?,
                &id.0,
                &PrimitiveDateTime::new(utc.date(), utc.time()),
                &uptime_u32,
                &population,
                &"azerust-0.1.0",
            )
            .await
            .map_err(|e| RealmListError::PersistError(e.to_string()))?;

        Ok(())
    }

    #[instrument(skip(self))]
    async fn create_realm(&self, name: &str, address: &IpAddr, local_address: &IpAddr, local_subnet_mask: &IpAddr, port: u16) -> Result<Realm, RealmListError> {
        realms::create_realm()
            .bind(
                &mut self.pool.get().await.map_err(|e| RealmListError::PersistError(e.to_string()))?,
                &name,
                address,
                local_address,
                local_subnet_mask,
                &port,
            )
            .map(|r| Realm {
                id: RealmId(r.id as u32),
                build: r.gamebuild,
                external_address: r.address,
                local_address: r.local_address,
                local_subnet_mask: r.local_subnet_mask,
                flags: r.flag,
                timezone: r.timezone,
                name: r.name.to_owned(),
                population: r.population,
                port: r.port,
                realm_type: match r.icon {
                    0 => RealmType::Normal,
                    1 => RealmType::PVP,
                    6 => RealmType::RP,
                    8 => RealmType::RPPvP,
                    _ => panic!(),
                },
            })
            .one()
            .await
            .map_err(|e| RealmListError::PersistError(e.to_string()))
    }
}
