#[derive(Debug)]
pub enum ClientPacket {
    AuthSession(AuthSession),
    KeepAlive,
    Ping { ping: u32, latency: u32 },
    ReadyForAccountDataTimes,
    CharEnum,
    RealmSplit { realm: u32 },
}

#[derive(Debug)]
pub struct AuthSession {
    pub build: u32,
    pub server_id: u32,
    pub username: String,
    pub login_server_type: u32,
    pub region_id: u32,
    pub local_challenge: [u8; 4],
    pub client_proof: [u8; 20],
    pub addons: Vec<Addon>,

    pub battlegroup_id: u32,
    pub realm_id: u32,
    pub dos_response: u64,
}

#[derive(Debug, Clone)]
pub struct Addon {
    pub name: String,
    signature: bool,
    pub crc: u32,
    crc2: u32,
}

impl Addon {
    pub fn new(name: String, signature: bool, crc: u32, crc2: u32) -> Self {
        Self {
            name,
            signature,
            crc,
            crc2,
        }
    }
}
