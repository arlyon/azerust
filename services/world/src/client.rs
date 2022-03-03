use azerust_game::accounts::AccountId;
use rand::{distributions::Standard, prelude::Distribution};

#[derive(Copy, Clone, Hash, PartialEq, Eq, Debug)]
pub struct ClientId(pub u64);

impl Distribution<ClientId> for Standard {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> ClientId {
        ClientId(rng.gen())
    }
}

/// contains identifying information about a connected client
#[derive(Copy, Clone)]
pub struct Client {
    pub id: ClientId,
    pub account: Option<AccountId>,
}
