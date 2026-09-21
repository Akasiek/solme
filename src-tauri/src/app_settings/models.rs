use serde::{Deserialize, Serialize};

macro_rules! app_setting_keys {
    ($($variant:ident => $key:literal),+ $(,)?) => {
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        pub(crate) enum AppSettingKey {
            $($variant),+
        }

        impl AppSettingKey {
            pub const fn as_str(self) -> &'static str {
                match self {
                    $(Self::$variant => $key),+
                }
            }

            pub fn parse(value: &str) -> Option<Self> {
                match value {
                    $($key => Some(Self::$variant),)+
                    _ => None,
                }
            }
        }
    };
}

app_setting_keys! {
    AwayLyricsEnabled => "lyrics.away_enabled",
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub away_lyrics_enabled: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            away_lyrics_enabled: true,
        }
    }
}

impl AppSettings {
    pub(crate) fn apply(&mut self, key: AppSettingKey, value: &str) -> Result<(), String> {
        match key {
            AppSettingKey::AwayLyricsEnabled => {
                self.away_lyrics_enabled = value.parse::<bool>().map_err(|error| {
                    format!(
                        "Invalid value for application setting {}: {error}",
                        key.as_str()
                    )
                })?;
            }
        }

        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(tag = "key", content = "value", rename_all = "camelCase")]
pub enum AppSettingUpdate {
    AwayLyricsEnabled(bool),
}

impl AppSettingUpdate {
    pub(crate) fn into_stored(self) -> (AppSettingKey, String) {
        match self {
            Self::AwayLyricsEnabled(enabled) => {
                (AppSettingKey::AwayLyricsEnabled, enabled.to_string())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::AppSettingUpdate;

    #[test]
    fn deserializes_tauri_setting_update_payload() {
        let update: AppSettingUpdate = serde_json::from_value(serde_json::json!({
            "key": "awayLyricsEnabled",
            "value": false
        }))
        .unwrap();

        assert_eq!(update, AppSettingUpdate::AwayLyricsEnabled(false));
    }
}
