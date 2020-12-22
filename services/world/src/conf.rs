use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct AuthServerConfig {
    bind_address: IpAddress,
    port: u32,

    character_database: String,
    login_database: String,
    world_database: String,

    realm_id: u32,
    data_dir: u32,
}
