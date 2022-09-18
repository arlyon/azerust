use async_graphql::Object;
use azerust_game::characters;

pub struct Character(pub characters::Character);

#[Object]
impl Character {
    async fn name(&self) -> &str {
        &self.0.name
    }
    async fn level(&self) -> u8 {
        self.0.level
    }
}
