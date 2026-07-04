use std::sync::{Arc, Mutex, RwLock};

use sqlx::SqlitePool;
use uuid::Uuid;

use crate::credentials::CredentialStore;

use super::{
    backend::MusicServer,
    models::{
        SavedServerProfile, ServerConnectionConfig, ServerInfo, ServerType, StoredServerProfile,
    },
    navidrome::NavidromeBackend,
    profile_store::ServerProfileStore,
};

pub struct MusicServerService {
    server: RwLock<Option<CurrentServer>>,
    profiles: ServerProfileStore,
    credentials: Arc<Mutex<Box<dyn CredentialStore>>>,
}

#[derive(Clone)]
struct CurrentServer {
    profile_id: String,
    backend: Arc<dyn MusicServer>,
}

impl MusicServerService {
    pub fn new(database: SqlitePool, credentials: Box<dyn CredentialStore>) -> Self {
        Self {
            server: RwLock::new(None),
            profiles: ServerProfileStore::new(database),
            credentials: Arc::new(Mutex::new(credentials)),
        }
    }

    pub async fn connect(&self, config: ServerConnectionConfig) -> Result<ServerInfo, String> {
        let ServerConnectionConfig {
            profile_id,
            server_type,
            url,
            username,
            password,
            save_credentials,
        } = config;

        let server = create_server(server_type, url.clone(), username.clone(), password.clone())?;
        let info = server.ping().await?;
        let profile_id = if save_credentials {
            self.save_profile(profile_id, server_type, url, username, password)
                .await?
        } else {
            Uuid::new_v4().to_string()
        };

        self.set_current_server(profile_id, server)?;
        Ok(info)
    }

    pub async fn connect_saved(&self, profile_id: Option<String>) -> Result<ServerInfo, String> {
        let (profile, password) = self.load_profile_with_password(profile_id).await?;
        let profile_id = profile.id.clone();
        let server = create_server(profile.server_type, profile.url, profile.username, password)?;
        let info = server.ping().await?;

        self.set_current_server(profile_id, server)?;
        Ok(info)
    }

    pub fn current_server(&self) -> Result<(String, Arc<dyn MusicServer>), String> {
        let current = self
            .server
            .read()
            .map_err(|_| "Music server state lock was poisoned".to_string())?
            .clone()
            .ok_or_else(|| "No music server is connected".to_string())?;
        Ok((current.profile_id, current.backend))
    }

    pub async fn cache_profile_id(&self) -> Result<Option<String>, String> {
        if let Some(current) = self
            .server
            .read()
            .map_err(|_| "Music server state lock was poisoned".to_string())?
            .as_ref()
        {
            return Ok(Some(current.profile_id.clone()));
        }

        self.profiles
            .load()
            .await
            .map(|profile| profile.map(|profile| profile.id))
    }

    pub async fn saved_profile(&self) -> Result<Option<SavedServerProfile>, String> {
        let profile = self.profiles.load().await?;

        let current_profile_id = self
            .server
            .read()
            .map_err(|_| "Music server state lock was poisoned".to_string())?
            .as_ref()
            .map(|current| current.profile_id.clone());

        Ok(profile.map(|profile| {
            let is_current = current_profile_id.as_deref() == Some(profile.id.as_str());
            SavedServerProfile {
                id: profile.id,
                server_type: profile.server_type,
                url: profile.url,
                username: profile.username,
                is_current,
            }
        }))
    }

    pub async fn saved_profiles(&self) -> Result<Vec<SavedServerProfile>, String> {
        let mut profiles = self.profiles.load_all().await?;

        let current_profile_id = self
            .server
            .read()
            .map_err(|_| "Music server state lock was poisoned".to_string())?
            .as_ref()
            .map(|current| current.profile_id.clone());
        Ok(profiles
            .drain(..)
            .map(|profile| {
                let is_current = current_profile_id.as_deref() == Some(profile.id.as_str());
                SavedServerProfile {
                    id: profile.id,
                    server_type: profile.server_type,
                    url: profile.url,
                    username: profile.username,
                    is_current,
                }
            })
            .collect())
    }

    pub async fn forget_saved_profile(&self, profile_id: Option<String>) -> Result<(), String> {
        let credentials = Arc::clone(&self.credentials);

        let deleted_profile_id =
            if let Some(profile) = self.profiles.delete(profile_id.as_deref()).await? {
                let profile_id = profile.id.clone();
                tauri::async_runtime::spawn_blocking(move || {
                    credentials
                        .lock()
                        .map_err(|_| "Credential store lock was poisoned".to_string())?
                        .delete(&profile_id)
                })
                .await
                .map_err(|error| format!("Failed to delete saved credentials: {error}"))??;
                Some(profile.id)
            } else {
                None
            };

        if let Some(deleted_profile_id) = deleted_profile_id {
            let mut current_server = self
                .server
                .write()
                .map_err(|_| "Music server state lock was poisoned".to_string())?;
            if current_server
                .as_ref()
                .is_some_and(|current| current.profile_id == deleted_profile_id)
            {
                *current_server = None;
            }
        }

        Ok(())
    }

    pub async fn ping(&self) -> Result<ServerInfo, String> {
        let (_, server) = self.current_server()?;
        server.ping().await
    }

    async fn save_profile(
        &self,
        profile_id: Option<String>,
        server_type: ServerType,
        url: String,
        username: String,
        password: String,
    ) -> Result<String, String> {
        let credentials = Arc::clone(&self.credentials);
        let id = match profile_id {
            Some(profile_id) => profile_id,
            None => self
                .profiles
                .load_all()
                .await?
                .into_iter()
                .find(|profile| {
                    profile.server_type == server_type
                        && profile.url == url
                        && profile.username == username
                })
                .map(|profile| profile.id)
                .unwrap_or_else(|| Uuid::new_v4().to_string()),
        };
        let profile = StoredServerProfile {
            id: id.clone(),
            server_type,
            url,
            username,
        };

        let credential_id = id.clone();
        tauri::async_runtime::spawn_blocking(move || {
            credentials
                .lock()
                .map_err(|_| "Credential store lock was poisoned".to_string())?
                .save(&credential_id, &password)
        })
        .await
        .map_err(|error| format!("Failed to save server credentials: {error}"))??;

        self.profiles.save(&profile).await?;
        Ok(id)
    }

    async fn load_profile_with_password(
        &self,
        profile_id: Option<String>,
    ) -> Result<(StoredServerProfile, String), String> {
        let credentials = Arc::clone(&self.credentials);

        let profile = match profile_id {
            Some(profile_id) => self
                .profiles
                .load_by_id(&profile_id)
                .await?
                .ok_or_else(|| format!("Server profile {profile_id} is not saved"))?,
            None => self
                .profiles
                .load()
                .await?
                .ok_or_else(|| "No server profile is saved".to_string())?,
        };
        let credential_id = profile.id.clone();
        let password = tauri::async_runtime::spawn_blocking(move || {
            let password = credentials
                .lock()
                .map_err(|_| "Credential store lock was poisoned".to_string())?
                .load(&credential_id)?;
            Ok::<String, String>(password)
        })
        .await
        .map_err(|error| format!("Failed to load saved credentials: {error}"))??;

        Ok((profile, password))
    }

    pub(crate) fn set_current_server(
        &self,
        profile_id: String,
        backend: Arc<dyn MusicServer>,
    ) -> Result<(), String> {
        let mut current_server = self
            .server
            .write()
            .map_err(|_| "Music server state lock was poisoned".to_string())?;
        *current_server = Some(CurrentServer {
            profile_id,
            backend,
        });
        Ok(())
    }
}

fn create_server(
    server_type: ServerType,
    url: String,
    username: String,
    password: String,
) -> Result<Arc<dyn MusicServer>, String> {
    match server_type {
        ServerType::Navidrome => Ok(Arc::new(NavidromeBackend::new(url, username, password)?)),
    }
}
