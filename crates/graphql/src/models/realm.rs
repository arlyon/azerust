use async_graphql::Object;
use azerust_game::realms;

pub struct Realm(pub realms::Realm);

#[Object]
impl Realm {
    async fn name(&self) -> &str {
        &self.0.name
    }
    async fn realm_type(&self) -> String {
        self.0.realm_type.to_string()
    }
    async fn build(&self) -> u32 {
        self.0.build
    }
    async fn ip(&self) -> &str {
        &self.0.external_address
    }
    async fn port(&self) -> u16 {
        self.0.port
    }
    async fn timezone(&self) -> u8 {
        self.0.timezone
    }
}
