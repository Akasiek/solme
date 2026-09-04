use std::time::{SystemTime, UNIX_EPOCH};

pub fn now_epoch_seconds() -> Result<i64, String> {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| format!("System clock is before Unix epoch: {error}"))?;
    i64::try_from(duration.as_secs()).map_err(|_| "System time exceeds i64 range".to_string())
}
