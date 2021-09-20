use super::Addon;

#[derive(Debug)]
pub enum ServerPacket {
    AuthResponse,
    AddonInfo(Vec<Addon>),
    ClientCacheVersion(u32),
    TutorialData,
    Pong(u32),
}
