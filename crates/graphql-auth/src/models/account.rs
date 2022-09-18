use async_graphql::Object;
use azerust_game::accounts;
use chrono::{DateTime, Utc};

pub struct Account(pub accounts::Account);

#[Object]
impl Account {
    async fn username(&self) -> &str {
        &self.0.username
    }
    async fn email(&self) -> &str {
        &self.0.email
    }
    async fn joindate(&self) -> &DateTime<Utc> {
        &self.0.joindate
    }
    async fn last_login(&self) -> &Option<DateTime<Utc>> {
        &self.0.last_login
    }
    async fn online(&self) -> bool {
        self.0.online != 0
    }
}
