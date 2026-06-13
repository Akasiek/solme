use std::time::{SystemTime, UNIX_EPOCH};

pub fn now_epoch_seconds() -> Result<i64, String> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .map_err(|error| format!("System clock is before Unix epoch: {error}"))
}
