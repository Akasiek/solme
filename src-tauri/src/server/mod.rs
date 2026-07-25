pub(crate) mod backend;
mod models;
mod navidrome;
mod profile_store;
mod query;
mod service;

pub(crate) use models::{AlbumQuery, ScrobbleEvent};
pub use models::{SavedServerEndpoint, SavedServerProfile, ServerConnectionConfig, ServerInfo};
pub use service::MusicServerService;
