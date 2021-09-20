use azerust_game::accounts::AccountId;

#[derive(Copy, Clone, Hash, PartialEq, Eq, Debug)]
pub struct ClientId(pub u64);

/// contains identifying information about a connected client
#[derive(Copy, Clone)]
pub struct Client {
    pub id: ClientId,
    pub account: Option<AccountId>,
}
