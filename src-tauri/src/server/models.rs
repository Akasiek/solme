use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerConnectionConfig {
    #[serde(default)]
    pub profile_id: Option<String>,
    pub server_type: ServerType,
    pub url: String,
    #[serde(default)]
    pub secondary_url: Option<String>,
    pub username: String,
    pub password: String,
    pub save_credentials: bool,
}

#[derive(Clone, Copy, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ServerType {
    Navidrome,
}

impl ServerType {
    pub(super) fn as_storage_value(self) -> &'static str {
        match self {
            ServerType::Navidrome => "navidrome",
        }
    }

    pub(super) fn from_storage_value(value: &str) -> Result<Self, String> {
        match value {
            "navidrome" => Ok(ServerType::Navidrome),
            value => Err(format!("Unknown server type in profile store: {value}")),
        }
    }
}

#[derive(Clone, Copy, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum SavedServerEndpoint {
    Primary,
    Secondary,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SavedServerProfile {
    pub id: String,
    pub server_type: ServerType,
    pub url: String,
    pub secondary_url: Option<String>,
    pub username: String,
    pub is_current: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerInfo {
    pub server_type: String,
    pub server_version: Option<String>,
    pub api_version: String,
    pub username: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScrobbleEvent {
    NowPlaying,
    Submission,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct StoredServerProfile {
    pub id: String,
    pub server_type: ServerType,
    pub url: String,
    pub secondary_url: Option<String>,
    pub username: String,
}

impl From<StoredServerProfile> for SavedServerProfile {
    fn from(profile: StoredServerProfile) -> Self {
        Self {
            id: profile.id,
            server_type: profile.server_type,
            url: profile.url,
            secondary_url: profile.secondary_url,
            username: profile.username,
            is_current: false,
        }
    }
}
