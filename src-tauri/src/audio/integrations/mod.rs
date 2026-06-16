#[cfg(target_os = "linux")]
mod mpris;

#[cfg(target_os = "linux")]
pub use mpris::start_mpris_service;
