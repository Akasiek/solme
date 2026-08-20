use std::{
    future::Future,
    sync::{Arc, Mutex, RwLock},
};

use sqlx::SqlitePool;
use uuid::Uuid;

use crate::credentials::CredentialStore;

use super::{
    backend::MusicServer,
    models::{
        SavedServerEndpoint, SavedServerProfile, ServerConnectionConfig, ServerInfo, ServerType,
        StoredServerProfile,
    },
    navidrome::{NavidromeBackend, ARTWORK_TRANSPORT_ERROR_PREFIX},
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

#[derive(Clone)]
struct ActiveServer {
    endpoint: SavedServerEndpoint,
    backend: Arc<dyn MusicServer>,
}

struct FailoverMusicServer {
    profile: StoredServerProfile,
    password: String,
    active: RwLock<ActiveServer>,
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
            secondary_url,
            username,
            password,
            save_credentials,
        } = config;
        let url = normalize_required_url(url)?;
        let secondary_url = normalize_optional_url(secondary_url);
        let should_update_password = !password.is_empty();
        let resolved_password = match (profile_id.as_deref(), should_update_password) {
            (_, true) => password.clone(),
            (Some(profile_id), false) => self.load_password(profile_id).await?,
            (None, false) => password.clone(),
        };

        let profile = StoredServerProfile {
            id: profile_id
                .clone()
                .unwrap_or_else(|| Uuid::new_v4().to_string()),
            server_type,
            url,
            secondary_url,
            username,
        };
        let (server, info) =
            create_connected_server(profile.clone(), resolved_password.clone()).await?;
        let profile_id = if save_credentials {
            self.save_profile(
                profile_id,
                server_type,
                profile.url,
                profile.secondary_url,
                profile.username,
                should_update_password.then_some(password),
            )
            .await?
        } else {
            profile.id
        };

        self.set_current_server(profile_id, server)?;
        Ok(info)
    }

    pub async fn connect_saved(&self, profile_id: Option<String>) -> Result<ServerInfo, String> {
        let (profile, password) = self.load_profile_with_password(profile_id).await?;
        let profile_id = profile.id.clone();
        let (server, info) = create_connected_server(profile, password).await?;

        self.set_current_server(profile_id, server)?;
        Ok(info)
    }

    pub async fn connect_saved_endpoint(
        &self,
        profile_id: Option<String>,
        endpoint: SavedServerEndpoint,
    ) -> Result<ServerInfo, String> {
        let (profile, password) = self.load_profile_with_password(profile_id).await?;
        let profile_id = profile.id.clone();
        let url = match endpoint {
            SavedServerEndpoint::Primary => profile.url.as_str(),
            SavedServerEndpoint::Secondary => profile
                .secondary_url
                .as_deref()
                .ok_or_else(|| "Saved server profile has no secondary URL".to_string())?,
        };
        let (backend, info) =
            connect_server_url(profile.server_type, url, &profile.username, &password).await?;
        let server = Arc::new(FailoverMusicServer::new(
            profile, password, endpoint, backend,
        ));

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
                secondary_url: profile.secondary_url,
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
                    secondary_url: profile.secondary_url,
                    username: profile.username,
                    is_current,
                }
            })
            .collect())
    }

    pub async fn forget_saved_profile(
        &self,
        profile_id: Option<String>,
    ) -> Result<Option<String>, String> {
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

        if let Some(deleted_profile_id) = deleted_profile_id.as_deref() {
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

        Ok(deleted_profile_id)
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
        secondary_url: Option<String>,
        username: String,
        password: Option<String>,
    ) -> Result<String, String> {
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
            secondary_url,
            username,
        };

        if let Some(password) = password {
            self.save_password(&id, password).await?;
        }

        self.profiles.save(&profile).await?;
        Ok(id)
    }

    async fn load_profile_with_password(
        &self,
        profile_id: Option<String>,
    ) -> Result<(StoredServerProfile, String), String> {
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
        let password = self.load_password(&profile.id).await?;

        Ok((profile, password))
    }

    async fn save_password(&self, profile_id: &str, password: String) -> Result<(), String> {
        let credentials = Arc::clone(&self.credentials);
        let credential_id = profile_id.to_string();

        tauri::async_runtime::spawn_blocking(move || {
            credentials
                .lock()
                .map_err(|_| "Credential store lock was poisoned".to_string())?
                .save(&credential_id, &password)
        })
        .await
        .map_err(|error| format!("Failed to save server credentials: {error}"))?
    }

    async fn load_password(&self, profile_id: &str) -> Result<String, String> {
        let credentials = Arc::clone(&self.credentials);
        let credential_id = profile_id.to_string();

        tauri::async_runtime::spawn_blocking(move || {
            let password = credentials
                .lock()
                .map_err(|_| "Credential store lock was poisoned".to_string())?
                .load(&credential_id)?;
            Ok::<String, String>(password)
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

impl FailoverMusicServer {
    fn new(
        profile: StoredServerProfile,
        password: String,
        endpoint: SavedServerEndpoint,
        backend: Arc<dyn MusicServer>,
    ) -> Self {
        Self {
            profile,
            password,
            active: RwLock::new(ActiveServer { endpoint, backend }),
        }
    }

    async fn connect(
        profile: StoredServerProfile,
        password: String,
    ) -> Result<(Arc<dyn MusicServer>, ServerInfo), String> {
        match connect_server_url(
            profile.server_type,
            &profile.url,
            &profile.username,
            &password,
        )
        .await
        {
            Ok((backend, info)) => Ok((
                Arc::new(Self::new(
                    profile,
                    password,
                    SavedServerEndpoint::Primary,
                    backend,
                )),
                info,
            )),
            Err(primary_error) => {
                let Some(secondary_url) = profile.secondary_url.as_deref() else {
                    return Err(primary_error);
                };

                log::warn!(
                    "Primary music server connection failed, trying secondary URL: {primary_error}"
                );
                let (backend, info) = connect_server_url(
                    profile.server_type,
                    secondary_url,
                    &profile.username,
                    &password,
                )
                .await?;
                Ok((
                    Arc::new(Self::new(
                        profile,
                        password,
                        SavedServerEndpoint::Secondary,
                        backend,
                    )),
                    info,
                ))
            }
        }
    }

    async fn with_failover<T, Fut, Operation>(&self, operation: Operation) -> Result<T, String>
    where
        Operation: Fn(Arc<dyn MusicServer>) -> Fut,
        Fut: Future<Output = Result<T, String>>,
    {
        self.with_conditional_failover(operation, |_| true).await
    }

    async fn with_artwork_failover<T, Fut, Operation>(
        &self,
        operation: Operation,
    ) -> Result<T, String>
    where
        Operation: Fn(Arc<dyn MusicServer>) -> Fut,
        Fut: Future<Output = Result<T, String>>,
    {
        self.with_conditional_failover(operation, |error| {
            error.starts_with(ARTWORK_TRANSPORT_ERROR_PREFIX)
        })
        .await
    }

    async fn with_conditional_failover<T, Fut, Operation, ShouldFailover>(
        &self,
        operation: Operation,
        should_failover: ShouldFailover,
    ) -> Result<T, String>
    where
        Operation: Fn(Arc<dyn MusicServer>) -> Fut,
        Fut: Future<Output = Result<T, String>>,
        ShouldFailover: Fn(&str) -> bool,
    {
        let active = self.active_server()?;

        if active.endpoint == SavedServerEndpoint::Secondary {
            match self.connect_endpoint(SavedServerEndpoint::Primary).await {
                Ok(primary) => {
                    return match operation(Arc::clone(&primary)).await {
                        Ok(value) => {
                            self.set_active_server(SavedServerEndpoint::Primary, primary)?;
                            Ok(value)
                        }
                        Err(error) if should_failover(&error) => {
                            log::warn!(
                            "Recovered primary music server request failed, returning to secondary URL: {error}"
                        );
                            operation(Arc::clone(&active.backend)).await
                        }
                        Err(error) => Err(error),
                    }
                }
                Err(error) => {
                    log::debug!("Primary music server is still unavailable: {error}");
                }
            }
        }

        match operation(Arc::clone(&active.backend)).await {
            Ok(value) => Ok(value),
            Err(error)
                if active.endpoint == SavedServerEndpoint::Primary && should_failover(&error) =>
            {
                if self.profile.secondary_url.is_none() {
                    return Err(error);
                }

                log::warn!("Primary music server request failed, trying secondary URL: {error}");
                let secondary = self
                    .connect_endpoint(SavedServerEndpoint::Secondary)
                    .await?;
                let value = operation(Arc::clone(&secondary)).await?;
                self.set_active_server(SavedServerEndpoint::Secondary, secondary)?;
                Ok(value)
            }
            Err(error) => Err(error),
        }
    }

    fn active_server(&self) -> Result<ActiveServer, String> {
        self.active
            .read()
            .map_err(|_| "Music server state lock was poisoned".to_string())
            .map(|active| active.clone())
    }

    async fn connect_endpoint(
        &self,
        endpoint: SavedServerEndpoint,
    ) -> Result<Arc<dyn MusicServer>, String> {
        let url = match endpoint {
            SavedServerEndpoint::Primary => self.profile.url.as_str(),
            SavedServerEndpoint::Secondary => self
                .profile
                .secondary_url
                .as_deref()
                .ok_or_else(|| "Saved server profile has no secondary URL".to_string())?,
        };
        let (backend, _) = connect_server_url(
            self.profile.server_type,
            url,
            &self.profile.username,
            &self.password,
        )
        .await?;

        Ok(backend)
    }

    fn set_active_server(
        &self,
        endpoint: SavedServerEndpoint,
        backend: Arc<dyn MusicServer>,
    ) -> Result<(), String> {
        let mut active = self
            .active
            .write()
            .map_err(|_| "Music server state lock was poisoned".to_string())?;
        *active = ActiveServer { endpoint, backend };
        Ok(())
    }
}

#[async_trait::async_trait]
impl MusicServer for FailoverMusicServer {
    async fn ping(&self) -> Result<ServerInfo, String> {
        self.with_failover(|server| async move { server.ping().await })
            .await
    }

    async fn library_revision(&self) -> Result<Option<String>, String> {
        self.with_failover(|server| async move { server.library_revision().await })
            .await
    }

    async fn artists(&self) -> Result<Vec<crate::library::models::Artist>, String> {
        self.with_failover(|server| async move { server.artists().await })
            .await
    }

    async fn albums(
        &self,
        query: super::models::AlbumQuery,
    ) -> Result<Vec<crate::library::models::Album>, String> {
        self.with_failover(|server| async move { server.albums(query).await })
            .await
    }

    async fn album(&self, id: &str) -> Result<crate::library::models::AlbumWithSongs, String> {
        self.with_failover(|server| async move { server.album(id).await })
            .await
    }

    async fn genres(&self) -> Result<Vec<crate::library::models::Genre>, String> {
        self.with_failover(|server| async move { server.genres().await })
            .await
    }

    async fn playback_uri(&self, song_id: &str) -> Result<String, String> {
        self.with_failover(|server| async move {
            server.ping().await?;
            server.playback_uri(song_id).await
        })
        .await
    }

    async fn scrobble(
        &self,
        song_id: &str,
        started_at_ms: i64,
        event: super::models::ScrobbleEvent,
    ) -> Result<(), String> {
        self.with_failover(
            |server| async move { server.scrobble(song_id, started_at_ms, event).await },
        )
        .await
    }

    async fn set_favorite(
        &self,
        item_kind: crate::library::models::LibraryItemKind,
        item_id: &str,
        favorite: bool,
    ) -> Result<(), String> {
        self.with_failover(|server| async move {
            server.set_favorite(item_kind, item_id, favorite).await
        })
        .await
    }

    async fn set_rating(
        &self,
        item_kind: crate::library::models::LibraryItemKind,
        item_id: &str,
        rating: Option<i64>,
    ) -> Result<(), String> {
        self.with_failover(
            |server| async move { server.set_rating(item_kind, item_id, rating).await },
        )
        .await
    }

    async fn album_artwork(
        &self,
        cover_art_id: &str,
    ) -> Result<Option<crate::library::models::BinaryArtwork>, String> {
        self.with_artwork_failover(|server| async move { server.album_artwork(cover_art_id).await })
            .await
    }

    async fn artist_artwork(
        &self,
        artist_id: &str,
    ) -> Result<Option<crate::library::models::BinaryArtwork>, String> {
        self.with_artwork_failover(|server| async move { server.artist_artwork(artist_id).await })
            .await
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

async fn create_connected_server(
    profile: StoredServerProfile,
    password: String,
) -> Result<(Arc<dyn MusicServer>, ServerInfo), String> {
    FailoverMusicServer::connect(profile, password).await
}

async fn connect_server_url(
    server_type: ServerType,
    url: &str,
    username: &str,
    password: &str,
) -> Result<(Arc<dyn MusicServer>, ServerInfo), String> {
    let server = create_server(
        server_type,
        url.to_string(),
        username.to_string(),
        password.to_string(),
    )?;
    let info = server.ping().await?;
    Ok((server, info))
}

fn normalize_required_url(url: String) -> Result<String, String> {
    let url = url.trim().to_string();
    if url.is_empty() {
        return Err("Server URL cannot be empty".to_string());
    }
    Ok(url)
}

fn normalize_optional_url(url: Option<String>) -> Option<String> {
    url.map(|url| url.trim().to_string())
        .filter(|url| !url.is_empty())
}
