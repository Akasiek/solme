use std::{
    path::PathBuf,
    sync::{Arc, Mutex, RwLock},
};

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
    profiles: Arc<Mutex<ServerProfileStore>>,
    credentials: Arc<Mutex<Box<dyn CredentialStore>>>,
}

#[derive(Clone)]
struct CurrentServer {
    profile_id: String,
    backend: Arc<dyn MusicServer>,
}

impl MusicServerService {
    pub fn new(profile_path: PathBuf, credentials: Box<dyn CredentialStore>) -> Self {
        Self {
            server: RwLock::new(None),
            profiles: Arc::new(Mutex::new(ServerProfileStore::new(profile_path))),
            credentials: Arc::new(Mutex::new(credentials)),
        }
    }

    pub async fn connect(&self, config: ServerConnectionConfig) -> Result<ServerInfo, String> {
        let ServerConnectionConfig {
            server_type,
            url,
            username,
            password,
            save_credentials,
        } = config;

        let server = create_server(server_type, url.clone(), username.clone(), password.clone())?;
        let info = server.ping().await?;
        let profile_id = if save_credentials {
            self.save_profile(server_type, url, username, password)
                .await?
        } else {
            Uuid::new_v4().to_string()
        };

        self.set_current_server(profile_id, server)?;
        Ok(info)
    }

    pub async fn connect_saved(&self) -> Result<ServerInfo, String> {
        let (profile, password) = self.load_profile_with_password().await?;
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

        let profiles = Arc::clone(&self.profiles);
        tauri::async_runtime::spawn_blocking(move || {
            profiles
                .lock()
                .map_err(|_| "Server profile store lock was poisoned".to_string())?
                .load()
                .map(|profile| profile.map(|profile| profile.id))
        })
        .await
        .map_err(|error| format!("Failed to load server profile: {error}"))?
    }

    pub async fn saved_profile(&self) -> Result<Option<SavedServerProfile>, String> {
        let profiles = Arc::clone(&self.profiles);
        let profile = tauri::async_runtime::spawn_blocking(move || {
            profiles
                .lock()
                .map_err(|_| "Server profile store lock was poisoned".to_string())?
                .load()
        })
        .await
        .map_err(|error| format!("Failed to load server profile: {error}"))??;

        Ok(profile.map(Into::into))
    }

    pub async fn forget_saved_profile(&self) -> Result<(), String> {
        let profiles = Arc::clone(&self.profiles);
        let credentials = Arc::clone(&self.credentials);

        tauri::async_runtime::spawn_blocking(move || {
            let profiles = profiles
                .lock()
                .map_err(|_| "Server profile store lock was poisoned".to_string())?;

            if let Some(profile) = profiles.load()? {
                credentials
                    .lock()
                    .map_err(|_| "Credential store lock was poisoned".to_string())?
                    .delete(&profile.id)?;
            }

            profiles.delete()
        })
        .await
        .map_err(|error| format!("Failed to forget saved server profile: {error}"))?
    }

    pub async fn ping(&self) -> Result<ServerInfo, String> {
        let (_, server) = self.current_server()?;
        server.ping().await
    }

    async fn save_profile(
        &self,
        server_type: ServerType,
        url: String,
        username: String,
        password: String,
    ) -> Result<String, String> {
        let profiles = Arc::clone(&self.profiles);
        let credentials = Arc::clone(&self.credentials);

        tauri::async_runtime::spawn_blocking(move || {
            let profiles = profiles
                .lock()
                .map_err(|_| "Server profile store lock was poisoned".to_string())?;
            let id = profiles
                .load()?
                .map(|profile| profile.id)
                .unwrap_or_else(|| Uuid::new_v4().to_string());
            let profile = StoredServerProfile {
                id: id.clone(),
                server_type,
                url,
                username,
            };

            credentials
                .lock()
                .map_err(|_| "Credential store lock was poisoned".to_string())?
                .save(&id, &password)?;
            profiles.save(&profile)?;
            Ok(id)
        })
        .await
        .map_err(|error| format!("Failed to save server profile: {error}"))?
    }

    async fn load_profile_with_password(&self) -> Result<(StoredServerProfile, String), String> {
        let profiles = Arc::clone(&self.profiles);
        let credentials = Arc::clone(&self.credentials);

        tauri::async_runtime::spawn_blocking(move || {
            let profile = profiles
                .lock()
                .map_err(|_| "Server profile store lock was poisoned".to_string())?
                .load()?
                .ok_or_else(|| "No server profile is saved".to_string())?;
            let password = credentials
                .lock()
                .map_err(|_| "Credential store lock was poisoned".to_string())?
                .load(&profile.id)?;

            Ok((profile, password))
        })
        .await
        .map_err(|error| format!("Failed to load saved credentials: {error}"))?
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
