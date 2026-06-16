#[derive(Clone, Debug, PartialEq)]
pub struct Preference {
    pub key: PreferenceKey,
    pub value: PreferenceValue,
}

impl Preference {
    pub fn volume(volume: f64) -> Self {
        Self {
            key: PreferenceKey::Volume,
            value: PreferenceValue::Volume(volume.clamp(0.0, 100.0)),
        }
    }

    pub fn volume_value(&self) -> Option<f64> {
        match self.value {
            PreferenceValue::Volume(volume) => Some(volume),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PreferenceKey {
    Volume,
}

impl PreferenceKey {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Volume => "volume",
        }
    }

    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "volume" => Ok(Self::Volume),
            _ => Err(format!("Unknown preference key: {value}")),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum PreferenceValue {
    Volume(f64),
}

impl PreferenceValue {
    pub fn serialize(&self) -> String {
        match self {
            Self::Volume(volume) => volume.clamp(0.0, 100.0).to_string(),
        }
    }

    pub fn parse(key: PreferenceKey, value: &str) -> Result<Self, String> {
        match key {
            PreferenceKey::Volume => value
                .parse::<f64>()
                .map(|volume| Self::Volume(volume.clamp(0.0, 100.0)))
                .map_err(|error| format!("Failed to parse volume preference: {error}")),
        }
    }
}
