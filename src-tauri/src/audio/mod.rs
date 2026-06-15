mod backend;
mod fader;
mod models;
#[cfg(target_os = "linux")]
mod mpris;
mod mpv;
mod player_service;
mod scrobble;

pub use models::{PlaybackState, PlayerStatus};
#[cfg(target_os = "linux")]
pub use mpris::start_mpris_service;
pub use mpv::MpvBackend;
pub use player_service::PlayerService;
pub(crate) use scrobble::ScrobbleRepository;
pub use scrobble::ScrobbleService;
