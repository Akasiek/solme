use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerConnectionConfig {
    pub server_type: ServerType,
    pub url: String,
    pub username: String,
    pub password: String,
    pub save_credentials: bool,
}

#[derive(Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ServerType {
    Navidrome,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SavedServerProfile {
    pub server_type: ServerType,
    pub url: String,
    pub username: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerInfo {
    pub server_type: String,
    pub server_version: Option<String>,
    pub api_version: String,
    pub username: String,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct StoredServerProfile {
    pub id: String,
    pub server_type: ServerType,
    pub url: String,
    pub username: String,
}

impl From<StoredServerProfile> for SavedServerProfile {
    fn from(profile: StoredServerProfile) -> Self {
        Self {
            server_type: profile.server_type,
            url: profile.url,
            username: profile.username,
        }
    }
}
